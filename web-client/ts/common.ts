// Common classes/functions for FeoBlog UI.

import bs58 from "bs58"
import * as commonmark from "commonmark"
import { DateTime } from "luxon";
import type { Writable } from "svelte/store";

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

export function markdownToHtml(markdown: string): string {
    let parsed = cmReader.parse(markdown)
    return cmWriter.render(parsed)
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
            console.error("Error in TaskTracker.run():", e)
            this.error(`Task threw an exception: ${e}`)
            throw e
        } finally {
            this._isRunning = false
            this.log(`Finished after ${timer}. Errors: ${this.errorCount} Warnings: ${this.warnCount}`) // calls notify()
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