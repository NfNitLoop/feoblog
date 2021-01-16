// Common classes/functions for FeoBlog UI.

import bs58 from "bs58"
import * as commonmark from "commonmark"

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

// Applies `filter` to up to `count` items before it begins yielding them.
// Useful for prefetching things in parallel with promises.
export function* prefetch<In, Out>(items: Iterable<In>, count: Number, filter: (In)=> Out): Generator<Out, void, undefined> {
    let outs: Out[] = []
    for (let item of items) {
        outs.push(filter(item))
        while (outs.length > count) {
            yield (outs.shift() as Out)
        }
    }

    while (outs.length > 0) {
        yield (outs.shift() as Out)
    }
}

export async function* prefetchAsync<In, Out>(items: AsyncIterable<In>, count: Number, filter: (In)=> Out): AsyncGenerator<Out, void, undefined> {
    let outs: Out[] = []
    for await (let item of items) {
        outs.push(filter(item))
        while (outs.length > count) {
            yield (outs.shift() as Out)
        }
    }

    while (outs.length > 0) {
        yield (outs.shift() as Out)
    }
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
