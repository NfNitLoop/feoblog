// Common classes/functions for FeoBlog UI.

import bs58 from "bs58"
import * as commonmark from "commonmark"
import { DateTime } from "luxon";
import type { Writable } from "svelte/store";
import nacl from "tweetnacl";

const USER_ID_BYTES = 32;
const PASSWORD_BYTES = USER_ID_BYTES + 4 // 4 bytes b58 checksum.

export const MAX_ITEM_SIZE = 32 * 1024 // 32k

// TODO: Deprecated. Use client.UserID.fromString() instead.
// Parse a base58 userID to a Uint8Array or throws a string error message.
export function parseUserID(userID: string): Uint8Array {
    if (userID.length == 0) {
        throw "UserID must not be empty."
    }

    let buf: Uint8Array;
    try {
        buf = bs58.decode(userID)
    } catch (error) {
        throw "Not valid base58"
    }

    if (buf.length < USER_ID_BYTES) {
        throw "UserID too short"
    }

    if (buf.length == PASSWORD_BYTES) {
        throw "UserID too long. (This may be a paswword!?)"
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
        return errorMessage
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
// use:fixLinks to just fix links to keep them inside the svelte-spa-router.
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


// Applies `asyncFilter` to up to `count` items before it begins yielding them.
// Useful for prefetching things in parallel with promises.
export async function* prefetch<T, Out>(items: AsyncIterable<T>, count: Number, asyncFilter: (t: T) => Promise<Out>): AsyncGenerator<Out> {
    let outs: Promise<Out>[] = []

    for await (let item of items) {
        outs.push(asyncFilter(item))
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
    debug(...data: any[]): void;
    error(...data: any[]): void;
    info(...data: any[]): void;
    log(...data: any[]): void;
    warn(...data: any[]): void;
}

export class ConsoleLogger implements Logger {

    error(...data: any[]): void {
        console.error(...data)

    }
    warn(...data: any[]): void {
        console.warn(...data)
    }

    // without this, only warn & error show:
    private debugEnabled = false

    debug(...data: any[]): void {
        if (this.debugEnabled) console.debug(...data)
    }
    info(...data: any[]): void {
        if (this.debugEnabled) console.info(...data)
    }
    log(...data: any[]): void {
        // I tend to treat this like a debug statement, so:
        if (this.debugEnabled) console.log(...data)
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
    private parent: TaskTracker|undefined
 
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
            return await asyncTask(this)
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
            return await subtask.run(taskName, asyncTask)
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

    warn(message: string) {
        this.writeLog({
            message,
            isWarning: true,
            timestamp: DateTime.local().valueOf()
        })
        this._warnCount += 1
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

type LogEntry = {
    timestamp: number
    message: string
    isError?: boolean
    isWarning?: boolean
    subtask?: TaskTracker
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
    let magitudes = ["bytes", "KiB", "MiB", "GiB", "TiB"]
    let count = bytes

    while (count > base && magitudes.length > 1) {
        count = count / base
        magitudes.shift()
    }

    // Show 3 significant digits:
    let out
    if (count < 10) {
        out = count.toFixed(2)
    } else if (count < 100) {
        out = count.toFixed(1)
    } else {
        out = count.toFixed(0)
    }

    return `${out} ${magitudes[0]}`
}

export function bytesToHex(bytes: Uint8Array): string {
    let out = []
    for (let byte of bytes) {
        out.push(byte.toString(16).padStart(2, "0"))
    }
    return out.join("")
}

// Wraps a (browser) File with some extra info.
export class FileInfo {
    readonly file: File
    readonly objectURL: string
    hash: Hash

    private constructor(file: File) {
        this.file = file
        this.objectURL = URL.createObjectURL(file)
    }

    static async from(file: File): Promise<FileInfo> {
        let fi = new FileInfo(file)
        
        // TODO: Not supported in Safari?
        let bytes = await file.arrayBuffer()
        let ui8a = new Uint8Array(bytes)
        fi.hash = Hash.ofBytes(ui8a)
        return fi
    }

    get name() { return this.file.name }
    get type() { return this.file.type }
    get size() { return this.file.size }

    get readableSize(): string {
        return readableSize(this.size)
    }

    private static supportedImagesTypes = [
        "image/jpeg",
        "image/png",
        "image/gif",
        "image/svg+xml",
    ]

    get isImage(): boolean {
        for (let type of FileInfo.supportedImagesTypes) {
            if (type === this.type) return true
        }
        return false
    }
}

// A 64-byte SHA-512 hash
export class Hash {
    readonly bytes: Uint8Array
    readonly asHex: string

    private constructor(hashBytes: Uint8Array, asHex: string) {
        this.bytes = hashBytes
        this.asHex = asHex
    }

    static ofBytes(bytes: Uint8Array): Hash {
        let hashBytes = nacl.hash(bytes)
        let asHex = bytesToHex(hashBytes)
        return new Hash(hashBytes, asHex)
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

    private async runQueue() {
        // already running:
        if (this.locked) return
        // Nothing to do:
        if (this.queue.length === 0) return

        this.setLocked(true)
        try {
            while (this.queue.length > 0) {
                let callback = this.queue.shift()!
                await callback()
            }
        } catch (e) {
            console.error("Exception in Mutex.runQueue()!?  Should be impossible.", e)
        } finally {
            this.setLocked(false)
        }
    }
}

