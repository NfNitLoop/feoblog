import { Item, ItemList, ItemListEntry, ItemType, Post } from "../protos/feoblog"
import * as nacl from "./naclWorker/nacl"
import tweetnacl from "tweetnacl"
import { bytesToHex, ConsoleLogger, Logger, prefetch } from "./common";
import { tick } from "svelte";
import { decodeBase58, decodeBase58Check, encodeBase58 } from "./fbBase58";

// Before sending files larger than this, we should check whether they exist:
const SMALL_FILE_THRESHOLD = 1024 * 128


// Encapsulates communication with the server(s).
export class Client {

    private base_url: string;

    constructor(config: Config) {
        this.base_url = config.base_url
    }

    // A human-readable representation of the server we're talking to. 
    get url() {
        return this.base_url || "local server"
    }

    // Load an Item from the server, if it exists.
    // Validates the signature of the Item before returning it.
    async getItem(userID: UserID|string, signature: Signature|string, options?: GetItemOptions): Promise<Item|null> {
        let bytes = await this.getItemBytes(userID, signature, options)
        if (bytes === null) return null
        return Item.deserialize(bytes)
    }

    // Like getItem(), but returns the item bytes so that the signature remains valid over the bytes.
    async getItemBytes(userID: UserID|string, signature: Signature|string, options?: GetItemOptions): Promise<Uint8Array|null> {
        
        // Perform validation of these before sending:
        if (typeof userID === "string") {
            userID = UserID.fromString(userID)
        }
        if (typeof signature === "string") {
            signature = Signature.fromString(signature)
        }

        let url = `${this.base_url}/u/${userID}/i/${signature}/proto3`
        let response = await fetch(url)

        if (response.status == 404) { return null }

        if (!response.ok) {
            throw `${url} response error: ${response.status}: ${response.statusText}`
        }
        let lengthHeader = response.headers.get("content-length")
        if (lengthHeader === null) {
            throw `The server didn't return a length for ${url}`
        }
        let length = parseInt(lengthHeader)
        if (length > LENIENT_MAX_ITEM_SIZE) {
            throw `${url} returned ${length} bytes! (max supported is ${LENIENT_MAX_ITEM_SIZE})`
        }
        if (length == 0) {
            throw `Got 0 bytes`
        }

        let buf = await response.arrayBuffer()
        let bytes = new Uint8Array(buf)

        // Note: This is a bit expensive when we're bulk-loading items.
        // But, if we don't check them when we load them from the server, what's the
        // point of having the signatures?
        if (!options?.skipSignatureCheck) {
            if (!await signature.isValid(userID, bytes)) {
                throw `Invalid signature for ${url}`
            }
        }

        return bytes
    }

    // Write an item to the server.
    // This assumes you have provided a valid userID & signature for the given bytes.
    async putItem(userID: UserID, signature: Signature, bytes: Uint8Array): Promise<Response> {
    
        let url = `${this.base_url}/u/${userID}/i/${signature}/proto3`
        
        let response: Response
        try {
            response = await fetch(url, {
                method: "PUT",
                body: bytes,
            })
            if (!response.ok) {
                throw `Error uploading Item: ${response.status} ${response.statusText}`
            }

        } catch (e) {
            console.error("PUT exception:", e)
            throw e
        }

        return response
    }

    async putAttachment(userID: UserID, signature: Signature, fileName: string, blob: Blob): Promise<void> {
        // If the file is already in the content store, we can save some bandwidth/time:
        if (blob.size > SMALL_FILE_THRESHOLD) {
            const {exists} = await this.headAttachment(userID, signature, fileName)
            if (exists) return
        }

        let url = this.attachmentURL(userID, signature, fileName)
        let response: Response
        try {
            response = await fetch(url, {
                method: "PUT",
                body: blob,
            })
            if (!response.ok) {
                throw `Error uploading attachment "${fileName}": ${response.status} ${response.statusText}`
            }

        } catch (e) {
            const {exists} = await this.headAttachment(userID, signature, fileName)
            if (exists) return // Someone beat us to the upload, everything's OK.
            // else:
            throw e
        }
    }

    private attachmentURL(userID: UserID, signature: Signature, fileName: string) {
        return `${this.base_url}/u/${userID}/i/${signature}/files/${fileName}`
    }

    async headAttachment(userID: UserID, signature: Signature, fileName: string): Promise<AttachmentMeta> {
        let url = this.attachmentURL(userID, signature, fileName)
        let response = await fetch(url, {
            method: "HEAD",
        })

        let exists = false
        if (response.status == 200) {
            exists = true
        } else if (response.status != 404) {
            throw `Unexpected response status: ${response.status}: ${response.statusText}`
        }

        let exceedsQuota = response.headers.get("X-FB-Quota-Exceeded") == "1"

        return { exists, exceedsQuota }
    }

    async getAttachment(userID: UserID, signature: Signature, fileName: string): Promise<ArrayBuffer|null> {
        let url = this.attachmentURL(userID, signature, fileName)
        let response = await fetch(url)
        if (response.status == 404) {
            return null
        }
        if (!response.ok) {
            throw `Error: ${response.status}: ${response.statusText}`
        }
        return await response.arrayBuffer()
    }

    // Like getItem, but just gets the latest profile that a server knows about for a given user ID.
    // The signature is returned in a header from the server. This function verifies that signature
    // before returning the Item.
    // We also verify that the Item has a Profile.
    async getProfile(userID: UserID|string): Promise<ProfileResult|null> {
        
        // Perform validation of these before sending:
        if (typeof userID === "string") {
            userID = UserID.fromString(userID)
        }

        let url = `${this.base_url}/u/${userID}/profile/proto3`
        let response = await fetch(url)

        if (response.status == 404) { return null }

        if (!response.ok) {
            throw `${url} response error: ${response.status}: ${response.statusText}`
        }
        let lengthHeader = response.headers.get("content-length")
        if (lengthHeader === null) {
            console.log("response:", response)
            throw `The server didn't return a length for ${url}`
        }
        let length = parseInt(lengthHeader)
        if (length > LENIENT_MAX_ITEM_SIZE) {
            throw `${url} returned ${length} bytes! (max supported is ${LENIENT_MAX_ITEM_SIZE})`
        }
        if (length == 0) {
            throw `Got 0 bytes`
        }

        let sigHeader = response.headers.get("signature")
        if (sigHeader === null || sigHeader === "") {
            throw `The server did not return a signature for ${url}`
        }
        let signature = Signature.fromString(sigHeader)

        let buf = await response.arrayBuffer()
        let bytes = new Uint8Array(buf)

        if (!await signature.isValid(userID, bytes)) {
            throw `Invalid signature for ${url}`
        }

        let item: Item
        try {
            item = Item.deserialize(bytes)
        } catch (exception) {
            throw `Error deserializing ${url}: ${exception}`
        }
        if (item.profile === null) {
            throw `Server returned an Item for ${url} that is not a Profile.`
        }
        return {item, signature, bytes}
    }

    async * getHomepageItems(): AsyncGenerator<ItemListEntry> {
        let before: number|undefined = undefined
        while (true) {

            let list: ItemList = await this.getItemList("/homepage/proto3", {before})

            if (list.items.length == 0) {
                // There are no more items.
                return
            }
    
            for (let entry of list.items) yield entry
            
            if (list.no_more_items) {
                return
            }
    
            before = list.items[list.items.length - 1].timestamp_ms_utc
        }
    }

    async * getUserFeedItems(userID: UserID, params?: ItemOffsetParams): AsyncGenerator<ItemListEntry> {
        let isBefore = typeof(params?.after) != "number"
        let offset = {...params}

        while (true) {

            let list: ItemList = await this.getItemList(`/u/${userID}/feed/proto3`, offset)
            if (!isBefore) {
                /// We want to iterate in chronological order for this case, but the server always
                /// gives reverse order.  Fix it:
                list.items.reverse()
            }

            if (list.items.length == 0) {
                // There are no more items.
                return
            }
    
            for (let entry of list.items) yield entry
            
            if (list.no_more_items) {
                return
            }
    
            if (isBefore) {
                offset.before = list.items[list.items.length - 1].timestamp_ms_utc
            } else {
                offset.after = list.items[list.items.length - 1].timestamp_ms_utc
            }
        }
    }

    // TODO: getUserItems, getUserFeedItems, getHomepageItems, could share more code. They're basically all
    // paginating through an ItemList endpoint.
    async * getUserItems(userID: UserID): AsyncGenerator<ItemListEntry> {
        let before: number|undefined = undefined
        while (true) {

            let list: ItemList = await this.getItemList(`/u/${userID}/proto3`, {before})

            if (list.items.length == 0) {
                // There are no more items.
                return
            }
    
            for (let entry of list.items) yield entry
            
            if (list.no_more_items) {
                return
            }
    
            before = list.items[list.items.length - 1].timestamp_ms_utc
        }
    }

    async * getReplyItems(userID: UserID, signature: Signature): AsyncGenerator<ItemListEntry> {
        yield* this.paginateItemList(`/u/${userID}/i/${signature}/replies/proto3`)
    }

    private async * paginateItemList(url: string) {
        let before: number|undefined = undefined
        while (true) {
            let list: ItemList = await this.getItemList(url, {before})
            if (list.items.length == 0) {
                // There are no more items.
                return
            }
    
            for (let entry of list.items) yield entry
            
            if (list.no_more_items) {
                return
            }
    
            before = list.items[list.items.length - 1].timestamp_ms_utc
        }
    }

    // itemsPath: relative path to the thing that yields an ItemsList, ex: /homepage/proto3
    // params: Any HTTP GET params we might send to that path for pagination.
    private async getItemList(itemsPath: string, params?: Record<string,string|number|undefined>): Promise<ItemList> {

        let url = this.base_url + itemsPath
        if (params) {
            let sp = new URLSearchParams()
            for (let [key, value] of Object.entries(params)) {
                if (value === undefined) continue
                sp.set(key, value.toString())
            }
            url = `${url}?${sp}`
        }

        let response = await fetch(url)
        if (!response.ok) {
            console.error(`non-OK response from ${url}`, response)
            throw `Invalid response: ${response.status}: ${response.statusText}`
        }

        let buf = await response.arrayBuffer()
        let bytes = new Uint8Array(buf)
        return ItemList.deserialize(bytes)
    }
}

// Should not specify both before and after.
export interface ItemOffsetParams {
    /** timestamp in ms utc before which we want to query for items */
    before?: number

    /** timestamp in ms utc after which we want to query for items */
    after?: number
}

export type AttachmentMeta = {
    // The attachment already exists at the target location.
    exists: boolean,
    // Sending the attachment would exceed the user's quota:
    exceedsQuota: boolean,
}

export type GetItemOptions = {
    // When syncing items from one server to another, the receiving server MUST 
    // perform the verification, so verifying in the client is redundant and slow.
    // Set this flag to skip it.
    skipSignatureCheck?: boolean
}

// When we load a profile, we don't know its signature until it's loaded.
// Return the signature w/ the Item:
export class ProfileResult {
    item: Item
    signature: Signature
    bytes: Uint8Array
}

export class Config {
    // The base URL of a feoblog server.
    // Ex: base_url = "https://fb.example.com:8080". or "" for this server.
    base_url: string
}

export class UserID {
    toString(): string {
        return this.asBase58
    }

    // A hex representation of the bytes:
    toHex(): string {
        return bytesToHex(this.bytes)
    }

    static fromString(userID: string): UserID {
        let valType = typeof(userID)
        if (valType !== "string") {
            throw new Error(`invalid userID string of type ${valType}`)
        }
        if (userID.length == 0) {
            throw "UserID must not be empty."
        }
    
        let buf: Uint8Array;
        try {
            buf = decodeBase58(userID)
        } catch (error) {
            throw "UserID not valid base58"
        }
    
        UserID.validateBytes(buf)
        return new UserID(buf, userID)
    }


    static tryFromString(userID: string): UserID|null {
        try {
            return UserID.fromString(userID)
        } catch (error) {
            return null
        }
    }

    private static validateBytes(bytes: Uint8Array) {
        if (bytes.length < USER_ID_BYTES) {
            throw "UserID too short"
        }
    
        if (bytes.length == PASSWORD_BYTES) {
            throw "UserID too long. (This may be a paswword!?)"
        }
    
        if (bytes.length > USER_ID_BYTES) {
            throw "UserID too long."
        }
    }

    static fromBytes(bytes: Uint8Array): UserID {
        UserID.validateBytes(bytes)
        return new UserID(bytes, encodeBase58(bytes))
    }

    private constructor(readonly bytes: Uint8Array, readonly asBase58: string) { }
}

export class Signature {
    readonly bytes: Uint8Array

    toString(): string {
        return this.asBase58
    }

    // Check that a signature is valid.
    isValid(userID: UserID, bytes: Uint8Array): Promise<boolean> {
        return nacl.sign_detached_verify(bytes, this.bytes, userID.bytes)
    }

    isValidSync(userID: UserID, bytes: Uint8Array): boolean {
        return nacl.sign_detached_verify_sync(bytes, this.bytes, userID.bytes)
    }

    static fromString(signature: string): Signature {
        if (signature.length == 0) {
            throw "Signature must not be empty."
        }
    
        let buf: Uint8Array;
        try {
            buf = decodeBase58(signature)
        } catch (error) {
            throw "Signature not valid base58"
        }
    
        return Signature.fromBytes(buf)
    }

    static tryFromString(userID: string): Signature|null {
        try {
            return Signature.fromString(userID)
        } catch {
            return null
        }
    }

    static fromBytes(bytes: Uint8Array): Signature {
        if (bytes.length < SIGNATURE_BYTES) {
            throw new Error("Signature too short")
        }
    
        if (bytes.length > SIGNATURE_BYTES) {
            throw new Error("Signature too long.")
        }
    
        return new Signature(bytes, encodeBase58(bytes))
    }

    private constructor(bytes: Uint8Array, readonly asBase58: string) {
        this.bytes = bytes
    }
}

export class PrivateKey {
    readonly userID: UserID;

    static fromBase58(privateKey: string) {

        // Error to display about the private key:
        let buf: Uint8Array;
        try {
            buf = decodeBase58(privateKey)
        } catch (error) {
            throw "Not valid base58"
        }

        // Secret is 32 bytes, + 4 for checked base58.
        if (buf.length < 36) {
            throw "Key is too short."
        }
        if (buf.length > 36) {
            throw "Key is too long."
        }

        try {
            buf = decodeBase58Check(privateKey)
        } catch (e) {
            throw "Invalid Key"
        }

        // Signing is not usually a bottleneck so just using current thread:
        let keypair = tweetnacl.sign.keyPair.fromSeed(buf);        

        return new PrivateKey(keypair, privateKey)
    }

    private constructor(private keyPair: tweetnacl.SignKeyPair, readonly asBase58: string) {
        this.userID = UserID.fromBytes(keyPair.publicKey)
    }

    signDetached(message: Uint8Array) {
        return tweetnacl.sign.detached(message, this.keyPair.secretKey)
    }
           
}

const USER_ID_BYTES = 32;
const SIGNATURE_BYTES = 64;
const PASSWORD_BYTES = USER_ID_BYTES + 4 // 4 bytes b58 checksum.

const MAX_ITEM_SIZE = 32 * 1024 // 32KiB
// Some servers may increase max item size? Eh, we'll be lenient in what we accept
// Though, we do want to protect against trying to load absolutely massive ones in the browser:
const LENIENT_MAX_ITEM_SIZE = 1024 * 1024 // 1 MiB



// Contains an item, and its userID/signature, required to properly display it:
export interface DisplayItem {
    item: Item
    userID: UserID
    signature: Signature
}

interface DisplayItemData extends DisplayItem {
    itemData: Uint8Array
}

export type LazyItemLoaderOptions = {
    client: Client,
    
    /** Whether we should continue loading more items. */
    continueLoading: () => boolean,

    // A function we'll call when a new item is available to display:
    displayItem: (nextItem: DisplayItem) => Promise<void>,

    // A function we'll call when we've reached the end of the available items.
    endReached: () => void,

    itemEntries: AsyncGenerator<ItemListEntry>,

    itemFilter?: ItemFilter,

    log?: Logger,
}

// Many pages deal with lazily loading a list of items from an ItemList provided by the server.
// This class assists in that.
export class LazyItemLoader {

    constructor(options: LazyItemLoaderOptions) {
        this.client = options.client
        this.continueLoading = options.continueLoading
        this.displayItemCallback = options.displayItem
        this.lazyDisplayItems = prefetch(options.itemEntries, 4, this.fetchDisplayItem)
        this.endReached = options.endReached
        this.log = options.log || new ConsoleLogger()
        this.itemFilter = options.itemFilter || ItemFilter.allowAll()
    }

    private client: Client
    private continueLoading: () => boolean
    private displayItemCallback: (nextItem: DisplayItem) => Promise<void>
    private lazyDisplayItems: AsyncGenerator<DisplayItem|null>
    private endReached: () => void
    private log: Logger
    private itemFilter: ItemFilter

    #hasMore = true
    get hasMore() { return this.#hasMore && !this.stopped }
    
    // We've been stopped, and will never yield more items. Can drop in-progress items.
    private stopped = false

    private minItemsToDisplay = 10

    // Are we currently in the middle of displaying more items?
    private displayingMoreItems = false

    // Call this whenever the UI needs for us to display more items.
    // We'll continue displaying items until !continueLoading (at least).
    displayMoreItems = async () => {
        if (!this.hasMore) { return }
        if (this.displayingMoreItems) {
            // The user could scroll to the end of the page while we're still loading and displaying more items.
            // No need to fire off another async process:
            return
        }
        try {
            this.displayingMoreItems = true
            await this.displayMoreItems2()
        } finally {
            this.displayingMoreItems = false
        }
    }

    private async displayMoreItems2() {
        const continueLoading = this.continueLoading
        const displayItem = this.displayItemCallback
        this.log.debug("displayMoreItems, continueLoading", continueLoading())

        let minToDisplay = this.minItemsToDisplay

        while(minToDisplay > 0 || continueLoading()) {

            let n = await this.lazyDisplayItems.next()
            if (this.stopped) {
                // abort! We're no longer the live LazyLoader.
                return
            }

            if (n.done) {
                this.#hasMore = false
                this.endReached()
                return
            }

            if (n.value === null) {
                // lazyDisplayItems already warned about this. Just skip:
                continue
            }


            await displayItem(n.value)
            minToDisplay--

            // TODO: Deprecate continueLoading() and remove this.
            // Wait for Svelte to apply state changes.
            // MAY cause continueLoading to toggle off, but at least in Firefox that
            // doesn't always seem to have happened ASAP.
            // I don't mind loading a few more items than necessary, though.
            await tick()  
        }
    }

    private fetchDisplayItem = async (entry: ItemListEntry): Promise<DisplayItem|null> => {
        const filter = this.itemFilter

        if (!filter.byItemType(entry.item_type)) return null

        if (!filter.byTimestampMS(entry.timestamp_ms_utc)) return null

        let userID = UserID.fromBytes(entry.user_id.bytes)
        if (!filter.byUserID(userID)) return null

        let signature = Signature.fromBytes(entry.signature.bytes)
        if (!filter.bySignature(signature)) return null


        let item: Item 
        let bytes: Uint8Array|null
        try {
            // Postpone validation, because we may filter this item before trying to display it:
            bytes = await this.client.getItemBytes(userID, signature, { skipSignatureCheck: true })
            if (bytes === null) {
                // TODO: Display some placeholder?
                // It does seem like an error, the server told us about the item, but doesn't have it?
                this.log.error("No such item", userID, signature)
                return null
            }
            item = Item.deserialize(bytes)
        } catch (e) {
            this.log.error("Error loading Item:", userID, signature, e)
            return null
        }

        if (!filter.byItem(item)) return null

        // Check signature:
        if (!await signature.isValid(userID, bytes)) {
            this.log.error("Invalid signature for Item", userID, signature)
            return null
        }

        return {
            item,
            signature: signature,
            userID: userID,
        }
    }

    stop() { this.stopped = true }
}

/**
 * Allows filtering items as we fetch them with a lazy loader.
 * Methods should return `true` to keep a particular Item.
 * 
 * Note: The base ItemFilter returns true for everything.
 */
export class ItemFilter {

    protected constructor() {}

    static allowAll(): ItemFilter { return new ItemFilter() }

    static matchAll(filters: ItemFilter[]) {
        return new MatchAllFilter(filters)
    }

    byItemType(item_type: ItemType): boolean { return true }
    byUserID(userID: UserID): boolean { return true }
    bySignature(signature: Signature): boolean { return true }
    byTimestampMS(timestampMS: number): boolean { return true }

    /** The slowest filter, called after the item has been loaded from the server. */
    byItem(item: Item): boolean { return true}
}


const REGEX_NON_LITERAL = /[^a-z0-9_" -]/ig
export class FindMatchingString extends ItemFilter {
    readonly pattern: RegExp

    constructor(search: string) {
        super()
        search = search.replace(REGEX_NON_LITERAL, (match) => `\\${match}`)
        this.pattern = new RegExp(search, "i")
    }

    byItem(item: Item): boolean {
        const content = item.post?.body || item.comment?.text
        if (typeof content != "string") {
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
    #excludedTypes: ItemType[]

    constructor(itemTypes: ItemType[]) {
        super()
        this.#excludedTypes = itemTypes
    }

    byItemType(itemType: ItemType): boolean {
        return !this.#excludedTypes.some(t => t == itemType)
    }
}

class MatchAllFilter implements ItemFilter {

    constructor(private filters: ItemFilter[]) {}

    byItemType(itemType: ItemType): boolean {
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
    byTimestampMS(timestampMS: number): boolean {
        for (const filter of this.filters) {
            if (!filter.byTimestampMS(timestampMS)) return false
        }
        return true
    }
    byItem(item: Item): boolean {
        for (const filter of this.filters) {
            if (!filter.byItem(item)) return false
        }
        return true
    }

}