import * as bs58 from "bs58"
import * as bs58c from "bs58check"

// While a Buffer in theory extends a Uint8Array, the google-protobuf library
// checks the constuctor of the object to make sure it's actually a Uint8Array.
// See: https://github.com/protocolbuffers/protobuf/issues/1319
// TODO: In theory fixed in v3.21 by https://github.com/protocolbuffers/protobuf-javascript/pull/100/commits/5e339a3dec254371039ea49b364d5047ad31d6c1


// This module wraps our base58(check) dependencies to fix that.

// Also, as a nice side-effect, it provides types for the bs58check methods we wrap.

function fixType(value: Uint8Array): Uint8Array {
    if (value.constructor != Uint8Array) {
        return new Uint8Array(value)
    }
    return value
}


export function decodeBase58(value: string): Uint8Array {
    return fixType(bs58.decode(value))
}

export function encodeBase58(bytes: Uint8Array): string {
    return bs58.encode(bytes)
}

export function decodeBase58Check(value: string): Uint8Array {
    return fixType(bs58c.decode(value))
}

export function encodeBase58Check(bytes: Uint8Array) {
    return bs58c.encode(bytes)
}