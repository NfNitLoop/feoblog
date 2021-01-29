// Classes for communicating between the browser and web workers

// A client for the browser to send messages to the worker.
// Expects a single response for each message sent to worker.
export class WorkerProxy {
    private worker: Worker

    // Hold the responders for the in-progress operation:
    private inProgress: Responders|undefined

    private queuedSends: QueuedSend[] =  []


    constructor(url: string, options?: WorkerOptions) {
        let worker = new Worker(url, options)
        worker.onmessage = (m) => this.onmessage(m)
        worker.onerror = (m) => this.onerror(m)

        this.worker = worker
    }

    // Send a message to the worker.
    // Returns a promise that gets resolved with the result of the resulting operation in the worker.
    async send(...args: any[]): Promise<any> {

        let responders: Responders
        let promise = new Promise((resolve, reject) => {
            responders = {resolve, reject}
        })

        let queuedSend = {args, responders: responders!}
        this.queuedSends.push(queuedSend)

        this.sendNext()
        return promise
    }

    // Send the next message in our queue to the Worker:
    private sendNext() {
        if (this.inProgress) return
        if (this.queuedSends.length == 0) return

        let next = this.queuedSends.shift()!

        this.inProgress = next.responders
        this.worker.postMessage(next.args)
    }

    private onmessage(m: MessageEvent<any>) {
        if (!this.inProgress) {
            console.error("Received message with nothing in progress!?")
            return
        }

        this.inProgress.resolve(m.data)
        this.inProgress = undefined
        this.sendNext()
    }

    private onerror(m: ErrorEvent) {
        if (!this.inProgress) {
            console.error("Received error with nothing in progress!?")
            return
        }

        this.inProgress.reject(m.message)
        this.inProgress = undefined
        this.sendNext()
    }
}

type Responders = {
    resolve: (value: unknown) => void
    reject: (reason?: any) => void
}

// A message we'll send to the Worker later.
type QueuedSend = {
    args: any[]
    responders: Responders
}