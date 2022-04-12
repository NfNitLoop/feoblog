/// Implement obfuscated, timed local storage.

import type { Writable } from "svelte/store"
import * as nacl from "tweetnacl"
import type { AppState, SavedLogin } from "./app"
import type { UserID } from "./client";

export const securityLevels: LevelInfo[] = [
    { 
        key: "insecure", 
        // Note: At least in Chromium on Windows, adding an emoji causes the line-height to change.
        // To avoid UI twitching, add an emoji to every name.
        name: "☠️ Insecure",
        pros: [
            "You don't have to enter your password."
        ],
        cons: [
            "Your private key is stored in the browser unencrypted, indefinitely.",
            "☠️ If an attacker gains access to your computer, or compromises your browser, it is trivial for them to get your private key."
        ],
    },
    {
        key: "weak-password",
        name: "⚠️ Weak Password",
        pros: [
            "Relaxes the password security requirements so you can use a simpler password."
        ],
        cons: [
            "⚠️ Weak passwords can be \"cracked\" by brute-force. An attacker could recover both your password and your private key."
        ],

    },
    {
        key: "strong-password-cached",
        name: "✅ Strong Password + Timeout",
        pros: [
            "After entering your password once, the browser remembers your private key for a short time.",
            "Makes it easier to make lots of posts/comments without having to re-enter your password every time.",
            "The browser re-locks your private key if you close the window/tab.",
        ],
        cons: [
            "An unencrypted version of your private key is easily accessible if your browser is compromised while it's still in memory."
        ]
    },
    {
        key: "strong-password",
        name: "✅ Strong Password",
        cons: [
            "Strong passwords are still difficult to remember and type.",
            "⚠️ TODO: Password strength checking not yet implemented.",
        ],
        pros: [
            "A strong password might be easier to remember than a private key.",
            "While the private key is stored in the browser, it's securely protected by a strong password."
        ]
    },

    { 
        key: "most-secure",
        name: "✅ Most Secure",
        pros: [
            "✅ This is the most secure option.",
            "This is the default and recommended option.",
            "The browser never stores your private key. (Recommended: Use a third-party password manager.)",
        ],
        cons: [
            "You must enter your private key every time you make a post or comment, or edit your profile."
        ]
    }
]

// TODO: Move to app?  Might have a circular dependency here.
export class SecurityManager {
    
    private insecureStore: ObfusatingStore;
    private passwordStore: PrefixedStorage;
    private tempStore: TimedStorage;

    constructor(
        private app: Writable<AppState>,
    ) {
        this.insecureStore = new ObfusatingStore(new PrefixedStorage(window.localStorage, `insecureKeyStore`));
        this.passwordStore = new PrefixedStorage(window.localStorage, `privateKeyStore`);
        this.tempStore = new TimedStorage(`timedKeyStore`)
    }

    currentLevel(login: SavedLogin): SecurityLevelKey {
        // let matches = this.app.savedLogins.filter((l) => l.userID.toString() == this.login.userID.toString())
        // if (matches.length != 1) {
        //     throw new Error(`Expected to find userid: ${this.login.userID} but found ${matches.length} matches`)
        // }

        if (this.insecureStore.getItem(login.userID.toString())) {
            return 'insecure'
        }




        // TODO:
        if (!("securityLevel" in login)) {
            // old/default behavior:
            return "most-secure"
        }

        throw new Error("TODO: implement me")
    }

    setInsecure(uid: string, privateKey: string) {
        this.forgetKeys(uid)
        this.insecureStore.setItem(uid, privateKey)
        this.updateApp()
    }

    private forgetKeys(uid: string) {
        this.insecureStore.removeItem(uid)
        this.passwordStore.removeItem(uid)
        this.tempStore.removeItem(uid)
    }

    private updateApp() {
        console.log("updating app")
        this.app.update((app) => app)
    }
}

interface LevelInfo {
    // Don't use integer keys in case we want to introduce something 
    key: SecurityLevelKey
    name: string
    pros: string[]
    cons: string[]
}

export type SecurityLevelKey = "insecure"| "most-secure"|"weak-password"|"strong-password"|"strong-password-cached"


interface BetterStorage extends Storage {
    // A more efficient (and handy!) way to list keys.
    keys: string[]

    // Aww, was hoping VSCode would show me warnings for these.
    /** @deprecated Don't use this, use `keys`. */
    key(index: number): string|null
}


/** Delegates to inner storage, prefixes all keys with a known prefix. Namespace-y! */
export class PrefixedStorage implements BetterStorage {

    constructor(private inner: Storage, readonly prefix: string) { }

    get length(): number {
        return this.matchingInnerKeys.length
    }

    clear(): void {
        let keys = this.matchingInnerKeys
        for (let key of keys) {
            this.inner.removeItem(key)
        }
    }
    getItem(key: string): string | null {
        return this.inner.getItem(this.prefixedKey(key))
    }

    /** Getting keys this way is slow. Prefer using .keys */
    key(index: number): string | null {
        // Can't think of a way to make this reliably efficient.
        // You can't rely on key ordering when `inner` might get changed by other calls.
        // Have to re-calculate this every time. Can be O(n^2), but hopefully N is small here?
    
        let innerKey = this.matchingInnerKeys[index]
        if (innerKey == null) { return null }

        return this.removePrefix(innerKey)
    }

    // BLEH, this breaks composability. Probably won't get used, but IS more efficient.
    get keys(): string[] {
        return this.matchingInnerKeys.map((key) => this.removePrefix(key))
    }

    removeItem(key: string): void {
        this.inner.removeItem(this.prefixedKey(key))
    }
    setItem(key: string, value: string): void {
        this.inner.setItem(this.prefixedKey(key), value)
    }

    private prefixedKey(key: string): string {
        return `${this.prefix}:${key}`
    }

    private removePrefix(innerKey: string): string {
        return innerKey.substring(this.prefix.length + 1)
    }

    private get matchingInnerKeys() {
        let keys = []
        let end = this.inner.length
        for (let i = 0; i < end; i++) {
            let key = this.inner.key(i)
            if (key?.startsWith(this.prefix)) {
                keys.push(key)
            }
        }
        return keys
    }
}


class ObfusatingStore implements BetterStorage {
    constructor(private inner: BetterStorage) {}

    get length() {
        return this.inner.length;
    }
    clear(): void {
        this.inner.clear()
    }

    key(index: number): string | null {
        return this.inner.key(index)
    }

    get keys() {
        return this.inner.keys
    }

    removeItem(key: string): void {
        this.inner.removeItem(key)
    }

    getItem(key: string): string | null {
        let innerValue = this.inner.getItem(key)
        if (innerValue == null) { return null }

        return this.deObfuscate(innerValue)
    }

    setItem(key: string, value: string): void {
        this.inner.setItem(key, this.obfuscate(value))
    }

    // Just using base64 for obfuscation. Anything more would be needlessly complex and just offer a false sense of security.
    private obfuscate(value: string): string {
        let bytes = encoder.encode(value)
        return Base64.encode(bytes)
    }

    private deObfuscate(innerValue: string): string {
        let bytes = Base64.decode(innerValue)
        return decoder.decode(bytes)
    }

}

// Interface *similar* to Storage, but takes care of (de)serializing JSON for you.
class JSONStorage<T> {

    constructor(private inner: BetterStorage, private converter: JSONConverter<T>) {}

    setItem(key: string, value: T) {
        this.inner.setItem(key, this.converter.toJSON(value))
    }

    getItem(key: string): T|null {
        let value = this.inner.getItem(key)
        if (value == null) { return null }

        return this.converter.fromJSON(value)
    }

    get keys() { return this.inner.keys }

    removeItem(key: string) {
        this.inner.removeItem(key)
    }
    clear() {
        this.inner.clear()
    }
}

interface JSONConverter<T> {
    fromJSON(json: string): T
    toJSON(t: T): string
}


/**
 * Timed storage in a window.sessionStore. Will clear its values after timer elapses, or browser is closed.
 * Values are obfuscated.  (encrypted, but the key is also present)
 */
export class TimedStorage {
    private storage: JSONStorage<TimedData>;

    constructor(storeName: string) {
        // Why use session storage?
        // * gets deleted when the page is closed.
        // * is saved across a page reload, which is especially handy during development.
        var storage: BetterStorage;
        storage = new PrefixedStorage(window.sessionStorage, storeName)
        storage = new ObfusatingStore(storage)
        this.storage = new JSONStorage(storage, TimedData)
    }

    setItem(key: string, value: string, timeoutMs: number) {
        this.storage.setItem(key, new TimedData(value, timeoutMs))
    }

    getItem(key: string): string|null {
        let data = this.storage.getItem(key)
        if (data == null) { return null }
        if (data.timedOut) {
            this.storage.removeItem(key)
        } else {
            data.refresh()
            this.storage.setItem(key, data)
        }

        return data.value
    }

    removeItem(key: string) {
        this.storage.removeItem(key)
    }

    pruneExpired() {
        for (let key of this.storage.keys) {
            let data = this.storage.getItem(key)!
            if (data.timedOut) {
                this.storage.removeItem(key)
                console.debug("Removed expired item", key)
            }
        }
    }
}


class TimedData {
    static fromJSON(json: string): TimedData {
        let data: TimedDataJSON = JSON.parse(json)
        let td = new TimedData(data.value, data.validMs)
        td.expiresAt = data.expiresAt
        return td
    }

    static toJSON(td: TimedData): string {
        let data: TimedDataJSON = {
            value: td.value,
            expiresAt: td.expiresAt,
            validMs: td.validMs,
        }
        return JSON.stringify(data)
    }

    expiresAt: number;

    constructor(
        public value: string,
        public validMs: number,
    ) {
        this.refresh()
    }

    get timedOut() {
        return this.expiresAt >= this.now()
    }

    refresh() {
        this.expiresAt = this.now() + this.validMs
    }

    private now() {
        return new Date().valueOf()
    } 
}


interface TimedDataJSON {
    // Date.valueOf()
    expiresAt: number

    // How long the value should stay valid. Can be used to reset above:
    validMs: number

    value: string
}

const encoder = new TextEncoder()
const decoder = new TextDecoder()


// TODO: This is probably overkill for obfuscating! 😅
// Handles the details of encrypting/decrypting String values using a scret key.
class SecretBoxKey {

    static random(): SecretBoxKey {
        return new SecretBoxKey(nacl.randomBytes(nacl.secretbox.keyLength))
    }

    static fromBase64(value: string) {
        return new SecretBoxKey(Base64.decode(value))
    }

    // TODO: fromPassword.

    toString(): string {
        return this.toBase64()
    }

    toBase64(): string {
        return Base64.encode(this.bytes)
    }

    private bytes: Uint8Array

    private constructor(bytes: Uint8Array) {
        const keyLen = nacl.secretbox.keyLength
        if (bytes.length != keyLen) {
            throw new Error(`Expected ${keyLen} bytes but got ${bytes.length}`)
        }
        this.bytes = bytes
    }

    // utf8 string -> encrypted, base64-encoded string.
    encryptString(value: string): string {
        const nonce = nacl.randomBytes(nacl.secretbox.nonceLength)
        const message = encoder.encode(value)
        const boxed = nacl.secretbox(message, nonce, this.bytes)

        // Prepend the nonce to the output:
        const out = new Uint8Array(nonce.length + boxed.length)
        out.set(nonce)
        out.set(boxed, nonce.length)
        return Base64.encode(out)
    }

    decryptString(value: string): string | null {
        const bytes = Base64.decode(value)
        const nonce = bytes.slice(0, nacl.secretbox.nonceLength)
        const box = bytes.slice(nonce.length)

        const message = nacl.secretbox.open(box, nonce, this.bytes)
        if (message == null) {
            return null
        }
        return decoder.decode(message)
    }
}


// Handles encoding (Uint8Array -> base64) and decoding (base64 -> uint8array)
// TODO: The Buffer polyfill does base64 encoding, probably more efficiently than this. Use that.
class Base64 {
    static encode(value: Uint8Array): string {
        // Grossss:  https://developer.mozilla.org/en-US/docs/Glossary/Base64#the_unicode_problem
        const binaryString = Array.from(value).map(byte => String.fromCodePoint(byte)).join("")
        return btoa(binaryString)
    }

    static decode(value: string): Uint8Array {
        const binaryString = atob(value)
        const data = new Uint8Array(binaryString.length)
        for (let i = 0; i < binaryString.length; i++) {
            data[i] = binaryString.charCodeAt(i)
        }
        return data
    }
}



