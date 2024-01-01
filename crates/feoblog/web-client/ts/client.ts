import {
    UserID,
    Signature,
    PrivateKey,
    Client,
    ItemOffsetParams,
    ProfileResult,
    AttachmentMeta,
    protobuf as pb,
} from "feoblog-client"

export {
    UserID,
    Signature,
    PrivateKey,
    Client,
    pb as protobuf,
}

export type { 
    AttachmentMeta,
    ItemOffsetParams,
    ProfileResult,
}

import { ConsoleLogger, Logger, Mutex, prefetch } from "./common";


// Bridge function for getting inner Item types.
// See: https://github.com/bufbuild/protobuf-es/issues/337
type Maybe<T> = T|null|undefined
export function getInner(item: Maybe<pb.Item>, field: "post"): pb.Post | undefined;
export function getInner(item: Maybe<pb.Item>, field: "profile"): pb.Profile | undefined;
export function getInner(item: Maybe<pb.Item>, field: "comment"): pb.Comment | undefined;
export function getInner(item: Maybe<pb.Item>, field: "post"|"profile"|"comment"): pb.Post | pb.Profile | pb.Comment | undefined {
    let it = item?.itemType
    if (it?.case == field) {
        return it.value
    }

    // TODO: Use something like OneOf<> here for a more generic solution:
    // https://dev.to/maxime1992/implement-a-generic-oneof-type-with-typescript-22em
}

/// --------------------------------------------------------------------------------------------------

// Contains an item, and its userID/signature, required to properly display it:
export interface DisplayItem {
    item: pb.Item
    userID: UserID
    signature: Signature
}

export type LazyItemLoaderOptions = {
    client: Client,

    source: AsyncGenerator<pb.ItemListEntry>,

    filter: ItemFilter,

    log?: Logger,
}
export class LazyItemLoader {
    // idea: we could make an itemFilter that counts how many items have been filtered out.
    // shows progress searching for items when it's slow.

    constructor(options: LazyItemLoaderOptions) {
        this.client = options.client
        this.itemFilter = options.filter
        this.lazyDisplayItems = prefetch(options.source, 4, i => this.fetchDisplayItem(i))
    }

    private client: Client
    private lazyDisplayItems: AsyncGenerator<DisplayItem|null>
    private itemFilter: ItemFilter
    private log = new ConsoleLogger({prefix:"LazyItemLoader2"})

    #done = false
    get done() { return this.#done }
    abort() { 
        this.#done = true
        // Kill the asyncgenerator:
        this.lazyDisplayItems.return(null)
    }

    #mutex = new Mutex()

    /**
     * Get up to the next `count` DisplayItems.
     * May return a shorter list if there are no more items to fetch.
     */
    async getNext(count: number): Promise<Array<DisplayItem>> {
        return await this.#mutex.run(async () => {
            return this.#getNext(count)
        })
    }

    async #getNext(count: number): Promise<Array<DisplayItem>> {
        let out: DisplayItem[] = []

        for (; count > 0; count--) {
            if (this.#done) { 
                // inside loop because it can be set externally via abort()
                return out
            }
            let next = await this.lazyDisplayItems.next()
            if (next.done) {
                this.#done = true
                return out
            }
            if (next.value == null) {
                count++ // this one didn't count
            } else {
                out.push(next.value)
            }
        }

        return out
    }

    // Returns null if the item was filtered out.
    private async fetchDisplayItem(entry: pb.ItemListEntry): Promise<DisplayItem|null> {
        const filter = this.itemFilter

        if (!filter.byItemType(entry.itemType)) return null

        if (!filter.byTimestampMS(entry.timestampMsUtc)) return null

        let userID = UserID.fromBytes(entry.userId!.bytes)
        if (!filter.byUserID(userID)) return null

        let signature = Signature.fromBytes(entry.signature!.bytes)
        if (!filter.bySignature(signature)) return null

        let item: pb.Item 
        let bytes: Uint8Array|null
        try {
            // Postpone validation, because we may filter this item before trying to display it:
            bytes = await this.client.getItemBytes(userID, signature, { skipSignatureCheck: true })
            if (bytes === null) {
                // TODO: Display some placeholder?
                // It does seem like an error, the server told us about the item, but doesn't have it?
                this.log.error("Server advertises, but doesn't have:", userID, signature)
                return null
            }
            item = pb.Item.fromBinary(bytes)
        } catch (e) {
            this.log.error("Error loading Item:", userID, signature, e)
            return null
        }

        if (!filter.byItem(item)) return null

        // Check signature:
        if (!await signature.isValid(userID, bytes)) {
            this.log.error("Invalid signature for Item. Filtering out.", userID, signature)
            return null
        }

        return {
            item,
            signature: signature,
            userID: userID,
        }
    }
}

/**
 * Allows filtering items as we fetch them with a lazy loader.
 * Returning `false` removes an item from the list.
 * 
 * Note: The base ItemFilter returns true for everything.
 */
export class ItemFilter {

    protected constructor() {}

    static allowAll(): ItemFilter { return new ItemFilter() }

    static matchAll(filters: ItemFilter[]): ItemFilter {
        return new MatchAllFilter(filters)
    }

    byItemType(itemType: pb.ItemType): boolean { return true }
    byUserID(userID: UserID): boolean { return true }
    bySignature(signature: Signature): boolean { return true }
    byTimestampMS(timestampMS: bigint): boolean { return true }

    // TODO: byEntry(entry: ItemEntry)

    /** The slowest filter, called after the item has been loaded from the server. */
    byItem(item: pb.Item): boolean { return true }
}


const REGEX_NON_LITERAL = /[^a-z0-9_" -]/ig
export class FindMatchingString extends ItemFilter {
    readonly pattern: RegExp

    constructor(search: string) {
        super()
        search = search.replace(REGEX_NON_LITERAL, (match) => `\\${match}`)
        this.pattern = new RegExp(search, "i")
    }

    byItem(item: pb.Item): boolean {
        let content: string
        let it = item.itemType
        if (it.case == "post") {
            content = it.value.body
        } else if (it.case == "comment") {
            content = it.value.text
        } else {
            // I expect only post/comments in this view, so if we come across
            // something else, we can't filter on it, but I don't want to hide it:
            return true
        }

        // TODO: Search titles?

        return !!this.pattern.exec(content)
    }

    toString() { return `FindMatchingString(${this.pattern})` }

}

export class SkipUsers extends ItemFilter {
    constructor(private userIDs: Set<string>) {
        super()
    }

    byUserID(userID: UserID) {
        return !this.userIDs.has(userID.toString())
    }

}

export class ExcludeItemTypes extends ItemFilter {
    #excludedTypes: pb.ItemType[]

    constructor(itemTypes: pb.ItemType[]) {
        super()
        this.#excludedTypes = itemTypes
    }

    byItemType(itemType: pb.ItemType): boolean {
        return !this.#excludedTypes.some(t => t == itemType)
    }
}

class MatchAllFilter implements ItemFilter {

    constructor(private filters: ItemFilter[]) {}

    byItemType(itemType: pb.ItemType): boolean {
        for (const filter of this.filters) {
            if (!filter.byItemType(itemType)) return false
        }
        return true
    }

    byUserID(userID: UserID): boolean {
        for (const filter of this.filters) {
            if (!filter.byUserID(userID)) return false
        }
        return true
    }
    bySignature(signature: Signature): boolean {
        for (const filter of this.filters) {
            if (!filter.bySignature(signature)) return false
        }
        return true
    }
    byTimestampMS(timestampMS: bigint): boolean {
        for (const filter of this.filters) {
            if (!filter.byTimestampMS(timestampMS)) return false
        }
        return true
    }
    byItem(item: pb.Item): boolean {
        for (const filter of this.filters) {
            if (!filter.byItem(item)) return false
        }
        return true
    }

}