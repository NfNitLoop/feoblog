// Common classes/functions for FeoBlog UI.

import bs58 from "bs58"

const USER_ID_BYTES = 32;
const PASSWORD_BYTES = USER_ID_BYTES + 4 // 4 bytes b58 checksum.

export const MAX_ITEM_SIZE = 32 * 1024 // 32k


// Parse a base58 userID to a Uint8Array or throws a string error messag.
export function parseUserID(userID: string): Uint8Array {
    if (userID.length == 0) {
        throw "UserID is required."
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
