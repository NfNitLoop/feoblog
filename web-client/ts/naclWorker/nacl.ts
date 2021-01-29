import * as nacl from "tweetnacl-ts"
import {WorkerProxy} from "./coms"

// Wraps tweetnacl functions in async versions that run in a WebWorker if available.

interface NaCl {
    sign_detached_verify(msg: Uint8Array, sig: Uint8Array, publicKey: Uint8Array): Promise<boolean>
}

// This version just calls the 
class InBrowser implements NaCl {
    async sign_detached_verify(msg: Uint8Array, sig: Uint8Array, publicKey: Uint8Array): Promise<boolean> {
        return nacl.sign_detached_verify(msg, sig, publicKey)
    }
}

// Proxy to a webworker.
class Proxy implements NaCl {

    private worker: WorkerProxy 

    constructor() {
        // Ewww, because of the way webworkers work, the URL is relative to the page that first
        // loaded the script. So we need to use an absolute path to make this always work.
        let workerURL = "/client/ts/naclWorker/worker.js"
        this.worker = new WorkerProxy(workerURL, {type: "module", name: "Async TweetNaCl"})
    }

    async sign_detached_verify(msg: Uint8Array, sig: Uint8Array, publicKey: Uint8Array): Promise<boolean> {
        let response = await this.worker.send("sign_detached_verify", msg, sig, publicKey)
        return response
    }
}

let proxy: NaCl
try {
    // Note: This does NOT catch errors loading the Worker's worker.js.
    // TODO: Work-around Firefox & Safari's lack of module support in web workers.
    proxy = new Proxy()
} catch (e) {
    console.error("Error starting web worker:", e)
    console.warn(
        "Your browser does not support WebWorkers."
        + " Crypto operations will be performed on the main thread, which may poorly affect performance."
    )
    proxy = new InBrowser()
}

export function sign_detached_verify(msg: Uint8Array, sig: Uint8Array, publicKey: Uint8Array): Promise<boolean> {
    return proxy.sign_detached_verify(msg, sig, publicKey)
}
