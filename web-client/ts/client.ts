import { Item, ItemList, Post } from "../protos/feoblog"
import bs58 from "bs58"
import * as nacl from "tweetnacl-ts"

// Encapsulates communication with the server(s).
export class Client {

    private base_url: String;

    constructor(config: Config) {
        this.base_url = config.base_url
    }

    // Load an Item from the server, if it exists.
    // Validates the signature of the Item before returning it.
    async getItem(userID: UserID|string, signature: Signature|string): Promise<Item|null> {
        
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

        if (!signature.isValid(userID, bytes)) {
            throw `Invalid signature for ${url}`
        }

        return Item.deserialize(bytes)
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

        if (!signature.isValid(userID, bytes)) {
            throw `Invalid signature for ${url}`
        }

        let item: Item
        try {
            item = Item.deserialize(bytes)
        } catch (exception) {
            throw `Error deserializing ${url}: ${exception}`
        }
        if (item.profile === null) {
            throw `Server returned n Item for ${url} that is not a Profile.`
        }
        return {item, signature}
    }

    // Load the latest profile from any server that hosts profiles for this user.
    async getLatestProfile(userID: UserID|string): Promise<ProfileResult|null> {
        let result = await this.getProfile(userID)
        if (result === null) { return result }

        // TODO: Walk the profile servers and find the most-recently-updated one.
        return result
    }

    async getHomepageItems(before?: Number): Promise<ItemList> {
        let url = `${this.base_url}/homepage/proto3`

        if (before) {
            url = `${url}?before=${before}`
        }

        let response = await fetch(url)
        if (!response.ok) {
            console.error("getHomePageItems response", response)
            throw `Invalid response: ${response.status}: ${response.statusText}`
        }

        let buf = await response.arrayBuffer()
        let bytes = new Uint8Array(buf)
        return ItemList.deserialize(bytes)
    }

}

// When we load a profile, we don't know its signature until it's loaded.
// Return the signature w/ the Item:
export class ProfileResult {
    item: Item
    signature: Signature
}

export class Config {
    // The base URL of a feoblog server.
    // Ex: base_url = "https://fb.example.com:8080". or "" for this server.
    base_url: string
}

export class UserID {
    readonly bytes: Uint8Array

    toString(): string {
        return bs58.encode(this.bytes)
    }

    static fromString(userID: string): UserID {
        if (userID.length == 0) {
            throw "UserID must not be empty."
        }
    
        let buf: Uint8Array;
        try {
            buf = bs58.decode(userID)
        } catch (error) {
            throw "UserID not valid base58"
        }
    
        return UserID.fromBytes(buf)
    }

    static fromBytes(bytes: Uint8Array): UserID {
        if (bytes.length < USER_ID_BYTES) {
            throw "UserID too short"
        }
    
        if (bytes.length == PASSWORD_BYTES) {
            throw "UserID too long. (This may be a paswword!?)"
        }
    
        if (bytes.length > USER_ID_BYTES) {
            throw "UserID too long."
        }
    
        return new UserID(bytes)
    }

    private constructor(bytes: Uint8Array) {
        this.bytes = bytes
    }

}

export class Signature {
    readonly bytes: Uint8Array

    toString(): string {
        return bs58.encode(this.bytes)
    }

    // Check that a signature is valid.
    isValid(userID: UserID, bytes: Uint8Array): boolean {
        return nacl.sign_detached_verify(bytes, this.bytes, userID.bytes)
    }

    static fromString(userID: string): Signature {
        if (userID.length == 0) {
            throw "Signature must not be empty."
        }
    
        let buf: Uint8Array;
        try {
            buf = bs58.decode(userID)
        } catch (error) {
            throw "Signature not valid base58"
        }
    
        return Signature.fromBytes(buf)
    }

    static fromBytes(bytes: Uint8Array): Signature {
        if (bytes.length < SIGNATURE_BYTES) {
            throw "Signature too short"
        }
    
        if (bytes.length > SIGNATURE_BYTES) {
            throw "Signature too long."
        }
    
        return new Signature(bytes)
    }

    private constructor(bytes: Uint8Array) {
        this.bytes = bytes
    }
}

const USER_ID_BYTES = 32;
const SIGNATURE_BYTES = 64;
const PASSWORD_BYTES = USER_ID_BYTES + 4 // 4 bytes b58 checksum.

const MAX_ITEM_SIZE = 32 * 1024 // 32KiB
// Some servers may increase max item size? Eh, we'll be lenient in what we accept
// Though, we do want to protect against trying to load absolutely massive ones in the browser:
const LENIENT_MAX_ITEM_SIZE = 1024 * 1024 // 1 MiB