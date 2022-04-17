/// Implement obfuscated, timed local storage.

import { PasswordMeter } from "password-meter";
import type { Writable } from "svelte/store"
import * as nacl from "tweetnacl"
import type { AppState, SavedLogin } from "./app"
import { PrivateKey, UserID } from "./client";

export interface SecurityManagerOptions {
    userID: string,

    privateKeyBase58Check?: string
    privateKeyPassword?: string

    rememberKeySecs?: number
}

export interface SecurityRating {
    // 0-100 (%)
    score: number

    details: string[]

    errors: string[]
}

export interface SecuritySettings {
    // Since we don't have the password handy, have to store this calculated security score:
    score: number

    hasEncryptedKey: boolean,
    hasUnencryptedKey: boolean,
    hasUnlockedKey: boolean,

    keyTimeoutSecs?: number

    // TODO: keyTimeoutRemaining: number
}

let passwordMeter = new PasswordMeter()


// TODO: Move to app?  Might have a circular dependency here.
export class SecurityManager {
    
    // Stores unencrypted (but obfuscated) passwords permanently. ☠️☠️
    private insecureStore: ObfusatingStore;

    // Stores encrypted passwords
    private secureStore: SecretStorage;

    // Stores unencrypted (but obfuscated) keys which have been "unlocked" by being used recently.
    private tempStore: TimedStorage;

    constructor(
        private writableApp: Writable<AppState>,
        private app: AppState,
    ) {
        this.insecureStore = new ObfusatingStore(new PrefixedStorage(window.localStorage, `insecureKeyStore`));
        this.secureStore = new SecretStorage(new PrefixedStorage(window.localStorage, `privateKeyStore`));
        this.tempStore = new TimedStorage(`timedKeyStore`)
    }

    calculateLevel(opts: SecurityManagerOptions): SecurityRating {
        let score = 100
        let details: string[] = []
        let errors: string[] = []

        let securityImpact = (pct: number) => {
            let maxScore = 100 - pct
            score = Math.min(score, maxScore)
        }

        let {privateKeyBase58Check, privateKeyPassword, rememberKeySecs} = opts
        let savePrivateKey = privateKeyBase58Check !== undefined
        let storeWithPassword = privateKeyPassword !== undefined
        if (privateKeyBase58Check !== undefined) {

            if (privateKeyBase58Check == "") {
                return {score: 0, details: [], errors: ["Private Key must not be empty."]}
            }

            let privateKey
            try {
                privateKey = PrivateKey.fromBase58(privateKeyBase58Check)
            } catch (e) {
                return {score: 0, details: [], errors: ["Invalid private key"]}
            }

            if (privateKey.userID.asBase58 != opts.userID) {
                return {score: 0, details: [], errors: ["Private key is not for this user ID"]}
            }
        }

        let saveTemporarily = rememberKeySecs !== undefined && rememberKeySecs > 0

        if (savePrivateKey) {
            if (storeWithPassword) {
                securityImpact(5)
                details.push("You'll be able to enter your password instead of your private key.")
    
                let result = passwordMeter.getResult(privateKeyPassword!)
                let score = result.score
                if (score < 80) {
                    securityImpact(100)
                }
                if (score < 100) {
                    securityImpact(90)
                }
                if (score < 120) {
                    securityImpact(60)
                    details.push("☠️ Weak Password. If your browser is compromised, a weak password can be brute-forced, revealing both your password and your private key.")
                } else if (score > 180) {
                    details.push("✅ Strong password")
                } else {
                    securityImpact(10)
                }
                // details.push(`Password score: ${result.score} ${result.status}`)
    
    
            } else {
                details.push("☠️ Without a password, storing your private key in the browser could allow it to be compromised.")
                securityImpact(100)
            }
        } 
        
        if (!savePrivateKey && !saveTemporarily) {
            details.push("✅ You'll enter your private key any time you need to save a post. This is the most secure option.")
        }
    
        if (saveTemporarily) {
            details.push(`Each time you use your key, it will be stored temporarily to use again.`)
            details.push(
                `⚠️ During this time your key can be easily recovered if your browser is compromised.`
            )
            details.push("Your temporarily-stored key will always be cleared when the browser or tab is closed.")
    
            securityImpact(5)
            if (rememberKeySecs! > 15 * 60) {
                securityImpact(10)
            }
            if (rememberKeySecs! > 60 * 60 * 24) {
                securityImpact(15)
            }
        }
    
        details.push("Regardless of security options, your private key and password are never shared with the server.")
        details.push("A server admin will never need to know your private key or password.")
        if (savePrivateKey) {
            details.push("Remember: Despite saving it here, you must save your private key in a secure location, like a password manager.")
        }

        let savedLogin = this.app.getSavedLogin(opts.userID)
        if (savedLogin == null) {
            return {score: 0, details: [], errors: ["That user ID is not among the saved logins."] }
        }

        return { score, details, errors }
    }

    applySettings(opts: SecurityManagerOptions) {
        let check = this.calculateLevel(opts)
        if (check.errors.length > 0) {
            throw new Error(`tried to apply bad settings: ${check.errors[0]}`)
        }

        // Start from scratch:
        this.forgetKeys(opts.userID)

        if (opts.privateKeyBase58Check != undefined) {
            let password = opts.privateKeyPassword
            if (password != undefined) {
                this.secureStore.setItem(password, opts.userID, opts.privateKeyBase58Check)
            } else {
                this.insecureStore.setItem(opts.userID, opts.privateKeyBase58Check)
            }

            // Also save key in temp storage, since we just "used" it:
            if (opts.rememberKeySecs && opts.rememberKeySecs > 0) {
                this.tempStore.setItem(opts.userID, opts.privateKeyBase58Check, opts.rememberKeySecs * 1000)
            }
        }

        let savedLogin = this.app.getSavedLogin(opts.userID)
        if (savedLogin == null) { throw new Error(`I thought we already checked savedLogin was not null...`) }
        savedLogin.rememberKeySecs = opts.rememberKeySecs
        savedLogin.securityScore = check.score
        this.app.updateSavedLogin(savedLogin)
        this.updateApp()


    }

    getSettings(userID: string): SecuritySettings {
        let login = this.app.getSavedLogin(userID)
        return {
            // If we haven't saved any info, then security is 100%:
            score: login?.securityScore || 100,

            keyTimeoutSecs: login?.rememberKeySecs,
            hasEncryptedKey: this.hasEncryptedKey(userID),
            hasUnencryptedKey: this.hasUnencryptedKey(userID),
            hasUnlockedKey: this.hasUnlockedKey(userID),
        }
    }

    /** True if the user stores a key, w/ or w/o password. */
    hasEncryptedKey(userID: string): boolean {
        return this.secureStore.hasItem(userID)
    }

    hasUnencryptedKey(userID: string): boolean {
        return this.insecureStore.getItem(userID) != null
    }

    hasUnlockedKey(userID: string): boolean {
        return this.tempStore.hasItem(userID) != null
    }

    private setInsecure(uid: string, privateKey: string) {
        this.forgetKeys(uid)
        this.insecureStore.setItem(uid, privateKey)
        this.updateApp()
    }

    private forgetKeys(uid: string) {
        this.insecureStore.removeItem(uid)
        this.secureStore.removeItem(uid)
        this.tempStore.removeItem(uid)
    }

    private updateApp() {
        this.writableApp.update((app) => app)
    }
}


interface BetterStorage extends Storage {
    // A more efficient (and handy!) way to list keys.
    readonly keys: string[]

    // Aww, was hoping VSCode would show me warnings for these.
    /** @deprecated Don't use this, use `keys`. */
    key(index: number): string|null
}

/** Like Storage, but requires a password to get/set keys. */
class SecretStorage {
    constructor(private inner: BetterStorage) {}

    hasItem(key: string): boolean {
        return this.inner.getItem(key) != null
    }

    get keys() { return this.inner.keys }

    getItem(password: string, key: string): string|null {
        let boxed = this.inner.getItem(key)
        if (boxed == null) { return null }

        let box = SecretBoxKey.fromPassword(password)
        return box.decryptString(boxed)
    }

    setItem(password: string, key: string, value: string) {
        let box = SecretBoxKey.fromPassword(password)
        let boxed = box.encryptString(value)
        this.inner.setItem(key, boxed)
    }

    removeItem(key: string) {
        this.inner.removeItem(key)
    }
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

    constructor(private storeName: string) {
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

    /** Checks if an item exists. Does NOT refresh its duration. */
    hasItem(key: string): boolean {
        let data = this.storage.getItem(key)
        if (data == null) { return false }
        if (this.prune(key, data)) { return false }
        return true
    }

    private prune(key: string, data: TimedData): boolean {
        if (data.timedOut) {
            console.debug("Pruning old key:", `${this.storeName}:${key}`)
            this.removeItem(key)
            return true
        }
        return false
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
            this.prune(key, data)
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
        return this.expiresAt <= this.now()
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


// Handles the details of encrypting/decrypting String values using a scret key.
class SecretBoxKey {

    static random(): SecretBoxKey {
        return new SecretBoxKey(nacl.randomBytes(nacl.secretbox.keyLength))
    }

    static fromBase64(value: string) {
        return new SecretBoxKey(Base64.decode(value))
    }

    static fromPassword(password: string): SecretBoxKey {
        // 64 bytes:
        let hash = nacl.hash(encoder.encode(password))
        // 32 bytes:
        let secret = hash.subarray(0, nacl.secretbox.keyLength)

        return new SecretBoxKey(secret)
    }

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



