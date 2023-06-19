// Common classes/functions for FeoBlog UI.

import * as commonmark from "commonmark"
import { DateTime } from "luxon";
import { tick } from "svelte";
import type { Writable } from "svelte/store";
import nacl from "tweetnacl";
import { decodeBase58 } from "./fbBase58";

const USER_ID_BYTES = 32;
const PASSWORD_BYTES = USER_ID_BYTES + 4 // 4 bytes b58 checksum.

export const MAX_ITEM_SIZE = 32 * 1024 // 32k

// TODO: Deprecated. Use client.UserID.fromString() instead.
// Parse a base58 userID to a Uint8Array or throws a string error message.
/** @deprecated */
export function parseUserID(userID: string): Uint8Array {
    if (userID.length == 0) {
        throw "UserID must not be empty."
    }

    let buf: Uint8Array;
    try {
        buf = decodeBase58(userID)
    } catch (error) {
        throw "Not valid base58"
    }

    if (buf.length < USER_ID_BYTES) {
        throw "UserID too short"
    }

    if (buf.length == PASSWORD_BYTES) {
        throw "UserID too long. (This may be a password!?)"
    }

    if (buf.length > USER_ID_BYTES) {
        throw "UserID too long."
    }

    return buf
}

export function parseUserIDError(userID: string): string {
    try {
        parseUserID(userID)
    } catch (errorMessage) {
        return `${errorMessage}`
    }
    return ""
}

const cmReader = new commonmark.Parser()
const cmWriter = new commonmark.HtmlRenderer({ safe: true})

type MarkdownToHtmlOptions = {
    stripImages?: boolean
    withPreview?: FileInfo[]

    // A base URL to prepend to relative URLs.
    relativeBase?: string
}

export function markdownToHtml(markdown: string, options?: MarkdownToHtmlOptions): string {
    if (!markdown) { return ""}

    let parsed = cmReader.parse(markdown)

    if (options?.stripImages) {
        stripImages(parsed)
    }

    if (options?.withPreview) {
        previewImages(parsed, options.withPreview)
    }

    fixRelativeLinks(parsed, options)

    return cmWriter.render(parsed)
}

// Information about a markdown text.
// Tries to not expose library-specific data structures (ex: Node)
type MarkdownInfo = {
    linkDestinations: Set<string>
    imageDestinations: Set<string>
    unlinkedRefs: Set<string>
}

export function getMarkdownInfo(markdown: string): MarkdownInfo {
    let parsed = cmReader.parse(markdown)

    let linkDestinations = new Set<string>()
    let imageDestinations = new Set<string>()
    let unlinkedRefs = new Set<string>()

    let refCollector: string[] = []

    let walker = parsed.walker()
    for (let event = walker.next(); event; event = walker.next()) {
        if (!event.entering) continue
        let node = event.node
        if (node.type == "link") {
            linkDestinations.add(node.destination!)
            continue
        }

        if (node.type == "image") {
            imageDestinations.add(node.destination!)
            continue
        }

        if (node.type != "text") { continue }
        // By a quirk of the commonmark parser, it will give us an AST for [foo bar] that looks like:
        // <text>[</text>
        // <text>foo bar</text>
        // <text>]</text>
        // This also means that that text is NOT linked. Unless it's for a case like [foo [bar] baz]
        // but whatever, ignoring that pathological case. :p
        if (node.literal == "[" || node.literal == "![") {
            refCollector = [node.literal]
            continue
        }
        if (refCollector.length == 0) { continue }

        if (node.literal == "]") {
            refCollector.push(node.literal)
            unlinkedRefs.add(refCollector.join(""))
            refCollector = []
            continue
        }

        refCollector.push(node.literal!)
    }


    return {linkDestinations, imageDestinations, unlinkedRefs}
}

function fixRelativeLinks(root: commonmark.Node, options?: MarkdownToHtmlOptions) {
    if (!(options?.relativeBase)) { return }

    let walker = root.walker()
    for (let event = walker.next(); event; event = walker.next()) {
        if (!event.entering) continue

        let node = event.node
        if (!(node.type == "image" || node.type == "link")) continue
        if (!node.destination) continue

        let url = node.destination
        if (url.startsWith("/") || url.indexOf("//") >= 0) {
            // absolute URLs do not get corrected:
            continue
        }

        node.destination = options.relativeBase + node.destination
    }
}

function stripImages(root: commonmark.Node) {
    let walker = root.walker()

    for (let event = walker.next(); event; event = walker.next()) {
        if (!event.entering) continue

        let image = event.node
        if (image.type != "image") continue

        let altText = image.title?.trim()
        let imageTitle: string = ""
        if (image.firstChild && image.firstChild.type == "text") {
            imageTitle = image.firstChild.literal?.trim() || ""
        }

        let link = new commonmark.Node("link")
        link.destination = image.destination

        let linkText
        if (imageTitle && altText) {
            // Use both:
            linkText = altText
            link.title = imageTitle
        } else {
            // Use the first one:
            linkText = imageTitle || altText || link.destination
        }
        let textNode = new commonmark.Node("text")
        textNode.literal = linkText
        link.appendChild(textNode)

        // Replace:
        image.insertBefore(link)
        image.unlink()
        walker.resumeAt(link)
    }
}

function previewImages(root: commonmark.Node, attachments: FileInfo[]) {
    if (attachments.length === 0) return

    // Map FileInfo to their relative ./file/* paths for fast lookup:
    let fileMap = new Map<string,FileInfo>()
    for (let fi of attachments) {
        let key = encodeURI(`files/${fi.name}`)
        fileMap.set(key, fi)
        // Also allow ./-prefixed relative paths:
        fileMap.set(`./${key}`, fi)
    }

    let walker = root.walker()
    for (let event = walker.next(); event; event = walker.next()) {
        if (!event.entering) continue

        let image = event.node
        if (image.type != "image") continue
        if (!image.destination) continue

        let newDestination = fileMap.get(image.destination)
        if (newDestination) {
            // Replace w/ an objectURL to view the attached file inline:
            // (also avoids unnecessary hits to the server).
            image.destination = newDestination.objectURL
        }
    }
}

type FixLinksParams = {
    // Default: "fix"
    mode?: "ignore"|"newWindow"|"fix"
}

// Svelte link fixer: 
// use:fixLinks={{mode:"ignore"}} to ignore all link clicks.
// use:fixLinks={{mode:"newWindow"}} to open all link clicks in new windows.
// use:fixLinks to just fix links to keep them inside the single-page application.
export function fixLinks(node: HTMLElement, params?: FixLinksParams): {} {

    let activeParams = params
    let onClick = (event: Event) => {

        let target = event.target as HTMLElement
        let anchor: HTMLAnchorElement|undefined = undefined
        let tag = target.tagName
    
        if (tag == "A") {
            anchor = (target as HTMLAnchorElement)
        } else if (tag == "IMG") {
            let parent = target.parentElement
            if (parent?.tagName == "A") {
                anchor = (parent as HTMLAnchorElement)
            }
        }

        if (!anchor) return        
        interceptLinkClick(event, anchor, activeParams)
    }

    // Capture events to intercept events to hrefs:
    let useCapture = true
    node.addEventListener("click", onClick, useCapture)

    return {
        update(params: FixLinksParams) {
            activeParams = params
        },
        destroy() {
            node.removeEventListener("click", onClick, useCapture)
        }
    }
}

interface ObservableParams {
    enteredPage?: () => void,
    leftPage?: () => void,
}

export function observable(node: HTMLElement, params?: ObservableParams) {
    if (!params?.enteredPage && !params?.leftPage) { return {} }

    let visible = false; // start false so we always fire an initial enteredPage

    let setVisibility = (nowVisible: boolean) => {
        if (visible === nowVisible) {
            return // Nothing to do.
        }

        visible = nowVisible
        if (nowVisible) {
            params?.enteredPage?.()
        } else {
            params?.leftPage?.()
        }
    }

    let observer = new IntersectionObserver((entries: IntersectionObserverEntry[]) => {
        // We only observe this one element, so this should always be here:
        let entry = entries[0]
        setVisibility(entry.isIntersecting)
    })
    observer.observe(node)

    return {
        destroy() {
            // One last notification that this element is no longer on the page:
            setVisibility(false)
            observer.disconnect()
        }
    }
}

// These patterns should have a # prepended so that they stay inside of the client:
const CLIENT_LINK_PATTERNS: RegExp[] = [
    /^\/$/,
    /^\/u\/[^\/]+\/?$/,
    /^\/u\/[^\/]+\/(feed|profile)$/,
    /^\/u\/[^\/]+\/i\/[^\/]+\/?$/,
];

function interceptLinkClick(event: Event, anchor: HTMLAnchorElement, params?: FixLinksParams) {

    if (params?.mode === "ignore") {
        event.preventDefault()
        return
    }

    // Note: can't use anchor.href, because that gets resolved to a full http://blah.com/wharrgarbl.
    // We want to know if this is a relative link.
    let href = anchor.getAttribute("href")
    if (!href) return

    let matches = false
    for (let pat of CLIENT_LINK_PATTERNS) { 
        if (pat.exec(href)) {
            matches = true
            break
        }
    }

    // Must come first
    if (params?.mode === "newWindow") {
        anchor.target = "_blank"
    }

    if (matches) {
        anchor.href = `#${href}`
    }
}


// Applies `mapper` to up to `count` items before it begins yielding them.
// Useful for prefetching things in parallel with promises.
// `items` is assumed to be reasonably fast relative to `mapper`.
// TODO: replace with better-iterators?
export async function* prefetch<T, Out>(items: AsyncIterable<T>, count: number, mapper: (t: T) => Promise<Out>): AsyncGenerator<Out> {
    let outs: Promise<Out>[] = []

    // We assume items.next() is (generally, relatively) fast, so we always get it:
    for await (let item of items) {
        // .. and then queue the next mapper call(s), but don't wait for them:
        outs.push(mapper(item))
        while (outs.length > count) {
            yield assertExists(outs.shift())
        }
    }

    while (outs.length > 0) {
        yield assertExists(outs.shift())
    }
}

// TypeScript doesn't know that we've done our own bounds checking on things like Array.shift()
// Assert that we have:
function assertExists<T>(value: T|undefined): T {
    return value as T
}

// A small subset of the Console interface
export interface Logger {
    debug(...data: unknown[]): void;
    error(...data: unknown[]): void;
    info(...data: unknown[]): void;
    log(...data: unknown[]): void;
    warn(...data: unknown[]): void;
}

export interface LoggerOptions {
    prefix?: string
}

export class ConsoleLogger implements Logger {

    #prefix?: string

    constructor(opts?: LoggerOptions) {
        this.#prefix = opts?.prefix
    }

    error(...data: unknown[]): void {
        console.error(...this.#prefixedData(data))

    }
    warn(...data: unknown[]): void {
        console.warn(...this.#prefixedData(data))
    }

    // without this, only warn & error show:
    private debugEnabled = false

    debug(...data: unknown[]): void {
        if (this.debugEnabled) console.debug(...this.#prefixedData(data))
    }
    info(...data: unknown[]): void {
        if (this.debugEnabled) console.info(...this.#prefixedData(data))
    }
    log(...data: unknown[]): void {
        // I tend to treat this like a debug statement, so:
        if (this.debugEnabled) console.log(...this.#prefixedData(data))
    }

    #prefixedData(data: unknown[]): unknown[] {
        if (this.#prefix) {
            return [this.#prefix, ...data]
        }

        return data
    }

    withDebug(): ConsoleLogger {
        this.debugEnabled = true
        return this
    }
}

// An exception wrapper for TaskTracker so we can stop redundant logging.
class TaskTrackerException {
    cause: any

    constructor(cause: any) {
        this.cause = cause
    }
}

// Tracks the progress of some long-running async task.
export class TaskTracker 
{
    // A store that will get updated every time this object changes
    store: Writable<TaskTracker>|null = null

    // A parent we may need to notify of changes.
    public parent: TaskTracker|undefined

    // Limit the display of "temporary" tasks that may be too verbose to keep.
    public maxTempTasks = 20

    // Promises to any running subtasks:
    private subtasks: Promise<unknown>[] = []
 
    private _isRunning = false
    get isRunning() { return this._isRunning }

    private _logs: LogEntry[] = []
    get logs(): ReadonlyArray<LogEntry> {
        return this._logs
    }

    private _errorCount = 0
    private _warnCount = 0
    get errorCount() { return this._errorCount }
    get warnCount() { return this._warnCount }

    private _hasRun = false
    // Has this ever been run()?  (Might still be running!)
    get hasRun(): boolean {
        return this._hasRun
    }


    name = "(Unnamed)"

    async run<T>(taskName: string, asyncTask: (tracker: TaskTracker) => Promise<T>): Promise<T> {
        this.clear()
        this.name = taskName
        this._isRunning = true
        this._hasRun = true
        this.notify()

        let timer = new Timer()
        try {
            let out: T = await asyncTask(this)

            // TODO: I've forgotten. Why is this here?
            await Promise.all(this.subtasks)
            
            return out
        } catch (e) {
            if (!(e instanceof TaskTrackerException)) {
                console.error("Error in TaskTracker.run():", e)
                this.error(`Task threw an exception: ${e}`)
                throw new TaskTrackerException(e)
            }
            throw e
        } finally {
            this._isRunning = false
            this.log(`Finished after ${timer}.`)
            if (this.warnCount > 0) this.log(`Warnings: ${this.warnCount}`) 
            if (this.errorCount > 0) this.log(`Errors: ${this.errorCount}`)
        }
    }

    async runSubtask<T>(taskName: string, asyncTask: (tracker: TaskTracker) => Promise<T>): Promise<T> {
        if (!this.isRunning) {
            throw "Cannot run a subtask while the main task is not running."
        }

        let subtask = new TaskTracker()
        subtask.parent = this
        let entry: LogEntry = {
            message: taskName,
            timestamp: DateTime.local().valueOf(),
            subtask,
        }
        this.writeLog(entry)
        
        try {
            let promise = subtask.run(taskName, asyncTask)
            this.subtasks.push(promise)
            return await promise
        } finally {
            if (subtask.errorCount > 0) {
                entry.isError = true
            } else if (subtask.warnCount) {
                entry.isWarning = true
            }
    
            this._errorCount += subtask.errorCount
            this._warnCount += subtask.warnCount    
        }
    }

    private notify() {
        if (this.store) this.store.set(this)
        if (this.parent) this.parent.notify()
    }

    clear() {
        if (this._isRunning) {
            throw "Can't clear while running!"
        }
        this._logs = []
        this._errorCount = 0
        this._warnCount = 0
        this._hasRun = false
        this.notify()
    }

    private writeLog(log: LogEntry) {
        this._logs.push(log)
        this.notify()
    }

    error(message: string) {
        this.writeLog({
            message,
            isError: true,
            timestamp: DateTime.local().valueOf()
        })
        this._errorCount += 1
    }

    log(message: string) {
        let log = {
            message,
            timestamp: DateTime.local().valueOf()
        }
        this.writeLog(log)
    }

    // "temporary" logs can be cleaned up if there are too many of them.
    logTemp(message: string) {
        this._logs.push({
            message,
            timestamp: DateTime.local().valueOf(),
            temp: true
        })
        this.collapseTemps()
        this.notify()
    }

    warn(message: string) {
        this.writeLog({
            message,
            isWarning: true,
            timestamp: DateTime.local().valueOf()
        })
        this._warnCount += 1
    }

    private collapseTemps() {
        let tempCount = this._logs.filter(l => l.temp).length
        if (tempCount <= this.maxTempTasks) return

        let rLogs = [... this._logs]
        rLogs.reverse()

        let tempsFound = 0
        rLogs = rLogs.filter(e => {
            if (!e.temp) return true
            tempsFound++
            return tempsFound <= this.maxTempTasks
        })
        rLogs.reverse()
        this._logs = rLogs
    }

}

// Quick way to show elapsed time.
class Timer {
    startTime: number
    endTime: number|undefined

    constructor() {
        this.startTime = DateTime.local().valueOf()
    }

    stop() {
        this.endTime = DateTime.local().valueOf()
    }

    get deltaMS() {
        if (this.endTime) {
            return this.endTime - this.startTime 
        }
        return DateTime.local().valueOf() - this.startTime
    }

    toString() {
        let delta = this.deltaMS
        if (delta < 500) {
            // ex: 137ms
            return `${delta}ms`
        } 
        
        let secs = delta / 1000
        
        if (secs < 10) {
            // ex: 8.3 seconds.
            return `${secs.toFixed(1)} seconds`
        }

        if (secs < 70) {
            return `${secs.toFixed(0)} seconds`
        }

        secs = Math.round(secs)
        let minutes = Math.floor(secs / 60)
        secs = secs % 60
        return `${minutes}m${secs}s`
    }
}

/**
 * Improved timer.
 * * Has a nice .cancel() method.
 * * Will get canceled if you try to start() a new task to replace it.
 */
 export class CancelTimer {
    delayMs = 5000

    private timer: number|null = null

    start(callback: () => unknown, delayMs?: number) {
        this.cancel()

        let delay = (delayMs !== undefined) ? delayMs : this.delayMs
        this.timer = setTimeout(callback, delay) as unknown as number
    }

    cancel() {
        if (this.timer) {
            clearTimeout(this.timer)
        }
        this.timer = null
    }
}


type LogEntry = {
    timestamp: number
    message: string
    isError?: boolean
    isWarning?: boolean
    subtask?: TaskTracker

    // This log can be considered temporary and deleted if there are too many.
    temp?: boolean
}


const serverURLPattern = /^(https?:\/\/[^/ ]+)$/
// Returns a non-empty error string if `url` is not a valid server URL.
export function validateServerURL(url: string): string {
    if (url === "") {
        return "" // Don't show error in the empty case.
    }

    let match = serverURLPattern.exec(url)
    if (match === null) {
        return "Invalid server URL format"
    }

    return ""
}

// Give a size in human-readable 
export function readableSize(bytes: number): string {
    let base = 1024
    let magnitudes = ["bytes", "KiB", "MiB", "GiB", "TiB"]
    let count = bytes

    while (count > base && magnitudes.length > 1) {
        count = count / base
        magnitudes.shift()
    }
    let magnitude = magnitudes[0]

    // Show 3 significant digits:
    let out
    if (magnitude === "bytes") {
        out = count
    }else if (count < 10) {
        out = count.toFixed(2)
    } else if (count < 100) {
        out = count.toFixed(1)
    } else {
        out = count.toFixed(0)
    }

    return `${out} ${magnitude}`
}

export function bytesToHex(bytes: Uint8Array): string {
    let out = []
    for (let byte of bytes) {
        out.push(byte.toString(16).padStart(2, "0"))
    }
    return out.join("")
}

// TODO: (#122) Don't trust the File object given to us by the browser. 
// Make our own File/Blob object from its bytes.
// I'm pretty sure Safari on iOS changed the bytes out from under me.  (see: #120)

interface FileInfoArgs {
    name: string
    hash: Hash
    blob: Blob
    mimeType?: string
}

/**
 * An in-memory copy of a File object, w/ its hash.
 */
export class FileInfo {
    readonly blob: Blob
    name: string
    readonly hash: Hash
    readonly mimeType?: string

    objectURL: string

    private constructor({name, hash, blob, mimeType}: FileInfoArgs) {
        this.blob = blob
        this.name = name
        this.hash = hash
        this.mimeType = mimeType,
        this.objectURL = URL.createObjectURL(blob)
    }

    static async from(file: File): Promise<FileInfo> {
        // #122: Immediately read all file bytes and metadata into our own objects.
        // The browser's File object can not be trusted long-term. 
        // 1) I think I caught it sending different bytes than I attached. (can't reproduce)
        // 2) If a file's bytes change after attaching, you can't read it anymore. Unepxected exception.
        // Better to get the exception at attach time than when we're done w/ a post and trying to upload it.
        
        let buf = await file.arrayBuffer()
        let blob = new Blob([buf], {type: file.type})
        return new FileInfo({
            name: file.name,
            blob,
            hash: Hash.ofBuf(buf),
            mimeType: file.type
        })
    }

    debug() {
        console.debug({
            file: this.name,
            size: this.blob.size,
            hash: this.hash.asHex
        })
        return this
    }

    get type() { return this.blob.type }
    get size() { return this.blob.size }

    get readableSize(): string {
        return readableSize(this.size)
    }

    private static supportedImagesTypes = new Set([
        "image/jpeg",
        "image/png",
        "image/gif",

        // ⚠️ Nope! SVG can include JavaScript. :(
        // "image/svg+xml",
    ])

    get isImage(): boolean {
        return FileInfo.supportedImagesTypes.has(this.type)
    }

    /** Cleanup the objectURL reference, to free up memory. */
    close() {
        if (this.objectURL != "") {
            URL.revokeObjectURL(this.objectURL)
            this.objectURL = ""
        }
    }
}

// A 64-byte SHA-512 hash
export class Hash {
    private constructor(readonly bytes: Uint8Array, readonly asHex: string) {}

    static ofBytes(bytes: Uint8Array): Hash {
        let hashBytes = nacl.hash(bytes)
        let asHex = bytesToHex(hashBytes)
        return new Hash(hashBytes, asHex)
    }

    static ofBuf(buf: ArrayBuffer): Hash {
        return Hash.ofBytes(new Uint8Array(buf))
    }
}

// Mutex to allow only one (async) process to proceed at a time.
// I'm surprised JavaScript doesn't have one of these built in already!? 
export class Mutex {
    private _locked = false
    private queue: (() => Promise<void>)[] = []
    private setLocked(locked: boolean) {
        this._locked = locked
        if (this.lockNotifier) this.lockNotifier(locked)
    }

    get locked(): boolean { return this._locked }

    // Can be set to a callback that gets called when .locked changes.
    lockNotifier: ((locked: boolean) => void)|undefined
    
    // Run a single callback with the locked mutex.
    // Will wait until a lock is available.
    run<T>(callback: () => Promise<T>): Promise<T> {
        let res: (value: T) => void
        let rej: (reason?: any) => void
        let promise = new Promise<T>((resolve, reject) => {
            res = resolve
            rej = reject
        })

        let myCallback = async () => {
            try {
                res(await callback())
            } catch (e) {
                rej(e)
            }
        }

        this.queue.push(myCallback)
        this.runQueue() // note: NO await

        return promise
    }

    // Only run this task if none is already running:
    runIfNone<T>(callback: () => Promise<T>): Promise<T>|null {
        if (this.locked) return null
        else return this.run(callback)
    }

    private async runQueue() {
        // already running:
        if (this.locked) return
        // Nothing to do:
        if (this.queue.length === 0) return

        this.setLocked(true)
        try {
            while (this.queue.length > 0) {
                let callback = this.queue.shift()!
                try {
                    await callback()
                } catch (_ignored) {
                    // Presumably, whatever else is waiting on this Promise
                    // will get the exception when they `await` it.
                    // No need for duplicate throw here.
                    // We have already waited for it to "complete".
                }
            }
        } catch (e) {
            console.error("Exception in Mutex.runQueue()!?  Should be impossible.", e)
        } finally {
            this.setLocked(false)
        }
    }
}

export interface InfiniteScrollParams {
    /** How many items we want to render, max. */
    maxItems?: number,
    /** How many items to trim when we hit maxItems. */
    trimBy?: number,
}



/**
 * Used for managing scroll events.
 */
 class ScrollState {
    constructor() {
        this.listen()
        // Longest I saw in ios safari smooth scrolling was 86ms.
        this.timer.delayMs = 200
    }

    private static onScroll: null|((e: Event) => void) = null
    private static onTouch: null|((e: Event) => void) = null

    private static touchStartEvents = ["touchstart", "touchmove", "touchforcechange"]
    private static touchEndEvents = ["touchend", "touchcancel"]
    private static touchEvents = [...ScrollState.touchStartEvents, ...ScrollState.touchEndEvents]

    private callbacks: (() => Promise<void>)[] = []
    private timer = new CancelTimer()
    private isScrolling = false
    private _isTouching = false

    #log = new ConsoleLogger({prefix: "ScrollState"}) //.withDebug()

    // Bleh: a lot of work to track whether the user is touching the screen.
    // However, touch events get canceled if you flick the screen for momentum-based scrolling, even if
    // you catch the scroll and start touching the screen again before scrolling finishes.
    private get isTouching() { return this._isTouching }
    private set isTouching(value) { 
        if (value != this._isTouching) {
            this.#log.debug("isTouching", value)
            this._isTouching = value
        }
    }

    /**
     * Scrolling activity is locked. Others should ignore onscroll events during this time.
     */
    private _isLocked = false
    get isLocked(): boolean { return this._isLocked }


    private listen() {
        if (ScrollState.onScroll) {
            // Only one ScrollState should be listening at a time. If a previous one leaked, clear it:
            window.removeEventListener("scroll", ScrollState.onScroll)
        }

        ScrollState.onScroll = (e) => this.windowScrolled(e)
        window.addEventListener("scroll", ScrollState.onScroll)

        if (ScrollState.onTouch) {
            for (const e of ScrollState.touchEvents) {
                window.removeEventListener(e, ScrollState.onTouch)
            }
        }

        ScrollState.onTouch = (e: Event) => this.touchEvent(e)

        for (const e of ScrollState.touchEvents) {
            window.addEventListener(e, ScrollState.onTouch)
        }
    }

    private touchEvent(e: Event) {
        if (this._isLocked) { return }

        if (ScrollState.touchStartEvents.includes(e.type)) {
            this.isTouching = true
            return
        }

        if (ScrollState.touchEndEvents.includes(e.type)) {
            this.isTouching = false
            this.runQueue()
            return
        }

        this.#log.warn("Unexpected event type:", e.type, e)
    }


    private windowScrolled(e: Event) {
        if (this._isLocked) {
            // Yeah, we know scroll events might happen here. Shouldn't re-trigger onQuiet callbacks.
            return
        }

        if (!this.isScrolling) {
            this.#log.debug("stared scrolling")
        }
        this.isScrolling = true
        this.timer.start(() => {this.scrollFinished()})
    }

    private scrollFinished() {
        this.#log.debug("scrollFinished()")
        this.isScrolling = false
        this.runQueue()
    }

    /**
     * Perform some action once scrolling has come to a stop.
     * Acts as an exclusive lock so that other events can't happen in the meantime.
     * If your callback returns true, it means content was added above the viewport
     * and we should scroll to adjust.
     */
    withQuietLock(callback: () => Promise<boolean>): Promise<boolean> {
        let res: (value: boolean) => void
        let rej: (reason?: any) => void
        let promise = new Promise<boolean>((resolve, reject) => {
            res = resolve
            rej = reject
        })

        let myCallback = async () => {
            try {
                res(await this.handleCallback(callback))
            } catch (e) {
                rej(e)
            }
        }

        this.callbacks.push(myCallback)
        this.runQueue() // note: NO await

        return promise
    }

    // Hmm, currently I think we don't use ScrollState to add things to the bottom of the page.
    // In theory, something might try to add to the bottom of the page while we're running withQuietLock(), which
    // would throw off the document length delta calculation.
    // TODO: We should add a "soft lock"(?) that allows adding things at the bottom of the page without waiting
    // for a quiet period, but that we can make sure isn't happening at the same time as the promises we run
    // during withQuietLock().
    // Orrrr see if there's just a simpler solution to this problem. 😅

    private async handleCallback(cb: () => Promise<boolean>): Promise<boolean> {
        this.#log.debug("handleCallback() ...")
        // Must base scroll deltas off of the beforeScroll, because changing content length will also change scroll position.
        let beforeScroll = window.scrollY
        let beforeLength = document.body.scrollHeight

        let shouldScroll = await cb()

        let afterLength = document.body.scrollHeight
        let lengthDelta = afterLength - beforeLength

        if (shouldScroll) {
            this.#log.debug(`before ${beforeLength} after: ${afterLength} delta: ${lengthDelta} shouldScroll: ${shouldScroll}`)
            window.scrollTo(window.scrollX, beforeScroll + lengthDelta)
        }

        return shouldScroll
    }

    private async runQueue() {
        // already running:
        if (this._isLocked) { return }
        // Still waiting:
        if (this.isScrolling) { return }
        if (this.isTouching) { return }
        // Nothing to do:
        if (this.callbacks.length === 0) { return }

        this.#log.debug("Running", this.callbacks.length, "callbacks")

        this._isLocked = true
        try {
            while (this.callbacks.length > 0) {
                let callback = this.callbacks.shift()!
                await callback()
            }
        } catch (e) {
            this.#log.error("Exception in runQueue()!?  Should be impossible.", e)
        } finally {
            this._isLocked = false
        }
        this.#log.debug("Done running callbacks")
    }
}

// A global singleton, so locks are shared.
export const scrollState = new ScrollState()

/**
 * Implements an infinite scrolling window.
 * You can pushBottom(T) or pushTop(T) to add items to the top or bottom of the list.
 * If the list goes over maxItems long, the other side will be trimmed.
 * 
 */
export class InfiniteScroll<T> {
    private maxItems: number
    private trimBy: number;
    #items: T[] = []

    #log = new ConsoleLogger({prefix: "InfiniteScroll"}) //.withDebug()

    #subscriptions: ((ts: T[]) => void)[] = []

    constructor(params?: InfiniteScrollParams) {
        this.maxItems = params?.maxItems ?? 200
        this.trimBy = params?.trimBy ?? 50
    }

    async pushBottom(item: T): Promise<void> {
        this.#log.debug("pushBottom")
        this.#log.debug("Before:", window.scrollY, document.body.scrollHeight)
        await this.push("bottom", item)
        this.#log.debug("After:", window.scrollY, document.body.scrollHeight)
    }

    async pushTop(item: T): Promise<void> {
        this.#log.debug("pushTop")
        await this.push("top", item)
    }

    clear() {
        this.#log.debug("clear()")
        this.#items = []
        this.notify()
    }

    private async push(where: "bottom"|"top", item: T): Promise<void> {
        let items = this.#items

        // simple (& hopefully common) case:
        if (where == "bottom" && this.#items.length < this.maxItems) {
            this.#items.push(item)
            this.notify()
            return
        }

        if (items.length >= this.maxItems) {
            this.#log.debug(`Adding to`, where, `triggered a trim`)
             // Trimming will change the length of the document. Need to do that separately from adding:
             await scrollState.withQuietLock(async () => {
                // console.debug("Trimming", where)
                let trimTop = where == "bottom"
                if (trimTop) {
                    items = items.slice(this.trimBy)
                } else {
                    items = items.slice(0, -this.trimBy)
                }
                this.#items = items
                this.notify()
                await tick()
                return trimTop
            })
        } 

        await scrollState.withQuietLock(async () => {
            let needsUpdate = false
            if (where == "bottom") {
                this.#items = [...items, item]
            } else {
                this.#items = [item, ...items]
                needsUpdate = true
            }
            this.notify()
            await tick()
            return needsUpdate
        })
    }


    // Implement Svelte's Store contract:
    subscribe(subscription: (value: T[]) => void): (() => void) {
        subscription(this.#items)
        this.#subscriptions.push(subscription)

        return () => {
            this.#subscriptions = this.#subscriptions.filter(s => s != subscription)
        }
    }

    private notify() {
        for (const sub of this.#subscriptions) {
            sub(this.#items)
        }
    }
}

export async function delayMs(timeout: number): Promise<void> {
    await new Promise((res) => {
        setTimeout(() => { res(null) }, timeout)
    })
}