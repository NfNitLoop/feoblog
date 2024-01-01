import * as nacl from "tweetnacl"
import {WorkerProxy} from "./coms"

// Thank youuuuu https://github.com/mitschabaude/esbuild-plugin-inline-worker
// @ts-ignore -- this gets wrapped by the above plugin.
import NaClWorker from "./nacl.worker.js"

// Wraps tweetnacl functions in async versions that run in a WebWorker if available.

interface NaCl {
    sign_detached_verify(msg: Uint8Array, sig: Uint8Array, publicKey: Uint8Array): Promise<boolean>
}

// This version just calls the function in the browser, synchronously.
// CPU-bound operations on the main thread make the UI seem unresponsive.
// Possible workaround: Defer all of these calculations until the UI goes "quiet", so that the renderer isn't blocked.
class InBrowser implements NaCl {
    async sign_detached_verify(msg: Uint8Array, sig: Uint8Array, publicKey: Uint8Array): Promise<boolean> {
        return nacl.sign.detached.verify(msg, sig, publicKey)
    }
}

// Proxy to a webworker.
class Proxy implements NaCl {

    private worker: WorkerProxy 

    constructor() {
        // Ewww, because of the way webworkers work, the URL is relative to the page that first
        // loaded the script. So we need to use an absolute path to make this always work.
        let workerURL = "/client/ts/naclWorker/worker.js"
        this.worker = new WorkerProxy(new NaClWorker())
    }

    async sign_detached_verify(msg: Uint8Array, sig: Uint8Array, publicKey: Uint8Array): Promise<boolean> {
        return await this.worker.send("sign_detached_verify", msg, sig, publicKey)
    }
}

let proxy: NaCl
if (window.Worker) {
    proxy = new Proxy()
} else {
    console.warn(
        "Your browser does not support WebWorkers."
        + " Crypto operations will be performed on the main thread, which may poorly affect performance."
    )
    proxy = new InBrowser()
}

export function sign_detached_verify(msg: Uint8Array, sig: Uint8Array, publicKey: Uint8Array): Promise<boolean> {
    return proxy.sign_detached_verify(msg, sig, publicKey)
}


export function sign_detached_verify_sync(msg: Uint8Array, sig: Uint8Array, publicKey: Uint8Array): boolean {
    return nacl.sign.detached.verify(msg, sig, publicKey)
}