import { DateTime, Duration, DurationLikeObject, DurationObjectUnits, ToRelativeOptions } from "luxon";
import { Readable, Subscriber, Unsubscriber, Writable, writable } from "svelte/store";
import { CancelTimer } from "./common";

/** Implements a Svelte Readable store that loads its value from a promise. */
export class AsyncStore<T> implements Readable<T> {

    private innerStore: Writable<T> = writable()

    constructor(promise: Promise<T>, initialValue: T, private errorValue?: T) {
        this.innerStore.set(initialValue)
        this.updateLaterWith(promise)
    }
 
    private async updateLaterWith(promise: Promise<T>) {
        try {
            let value = await promise
            this.innerStore.set(value)
        } catch (e) {
            if (this.errorValue) {
                this.innerStore.set(this.errorValue)
            } else {
                // we have no way to report that an error happened. To the console!
                console.error("AsyncStore caught unhandled exception:", e)
            }
        }
    }

    subscribe(run: Subscriber<T>): Unsubscriber {
        return this.innerStore.subscribe(run)
    }
}

export class PeriodicStore<T> implements Readable<T> {

    private unsubscribers: Map<symbol, Unsubscriber> = new Map()
    private timer = new CancelTimer()
    private inner: Writable<T>

    constructor(private provider: PeriodicProvider<T>) {
        let {value} = provider()
        this.inner = writable(value)
    }

    subscribe(subscriber: Subscriber<T>, invalidate?: (value?: T) => void): Unsubscriber {

        let key = Symbol()
        let unsub = this.inner.subscribe(subscriber, invalidate)
        this.unsubscribers.set(key, unsub)
        this.start()
        return () => { this.unsubscribe(key) }
    }

    // Start a timer in case there isn't one running.
    start() {
        this.timer.start(() => {this.poll()}, 0)
    }

    private poll() {
        let {value, nextPollMs} = this.provider()
        this.inner.set(value)

        if (nextPollMs > 0) {
            this.timer.start(() => {this.poll()}, nextPollMs)
        }
    }

    private unsubscribe(key: symbol) {
        this.unsubscribers.delete(key)
        if (this.unsubscribers.size == 0) {
            this.timer.cancel()
        }
    }
}

export interface PeriodicProvider<T> {
    (): PeriodicUpdate<T>
}

export interface PeriodicUpdate<T> {
    value: T

    // How many ms in the future we should schedule the next poll.
    nextPollMs: number
}

export class CountDown implements Readable<CountdownData> {

    private inner: PeriodicStore<CountdownData>
    private end: DateTime;

    constructor(end: Date) {
        this.end = DateTime.fromJSDate(end)
        this.inner = new PeriodicStore(() => this.provide())
    }

    private provide(): PeriodicUpdate<CountdownData> {
        let remainingMs = this.end.valueOf() - (new Date().valueOf())

        let nextPollMs = 60 * 60_000 // 1h

        let opts: Opts = {
            significantUnits: 2,
            units: ["days", "hours", "minutes"]
        }

        if (remainingMs < 24*60*60_000) { nextPollMs = 60_000 } 
        if (remainingMs < 60*60_000) { 
            nextPollMs = 1_000
            // For minutes, I really don't want to see seconds ticking away:
            opts.units = ["minutes"]
        }
        if (remainingMs < 2 * 60_000) {
            // OK now show seconds:
            opts.units = ["minutes", "seconds"]
        }
        if (remainingMs < 0) { nextPollMs = -1 }

        let remaining = relativeDuration(Duration.fromMillis(remainingMs), opts)

        return {
            nextPollMs,
            value: {
                remainingMs,
                endRelative: remaining
            }
        }
    }

    subscribe(run: Subscriber<CountdownData>, invalidate?: (value?: CountdownData) => void): Unsubscriber {
        return this.inner.subscribe(run, invalidate)
    }
}

interface CountdownData {
    remainingMs: number

    // The end-time in human-readable form relative to now:
    endRelative: string
}

// Svelte store that shows relative elapsed time.
export class ElapsedTime implements Readable<ElapsedData> {

    private inner: PeriodicStore<ElapsedData>
    private begin: DateTime;

    constructor(begin?: Date) {
        this.begin = DateTime.fromJSDate(begin ?? new Date())
        this.inner = new PeriodicStore(() => this.provide())
    }

    private provide(): PeriodicUpdate<ElapsedData> {
        let elapsedMs = (new Date().valueOf()) - this.begin.valueOf() 
        let nextPollMs = 1000 // ms

        let opts: Opts = {
            significantUnits: 2,
            units: ["days", "hours", "minutes", "seconds"]
        }

        if (elapsedMs > 60_000) { 
            nextPollMs = 60_000 
            opts.units = ["days", "hours", "minutes"]
        } 

        let elapsed = relativeDuration(Duration.fromMillis(-elapsedMs), opts)

        return {
            nextPollMs,
            value: {
                elapsedMs,
                elapsedRelative: elapsed
            }
        }
    }

    // Restart the elapsed timer from now.
    startNow() {
        this.begin = DateTime.fromJSDate(new Date())
        this.inner.start()
    }

    subscribe(run: Subscriber<ElapsedData>, invalidate?: (value?: ElapsedData) => void): Unsubscriber {
        return this.inner.subscribe(run, invalidate)
    }
}

export interface ElapsedData {
    elapsedMs: number
    elapsedRelative: string
}

// See: https://github.com/moment/luxon/issues/1129
// General solution for relative durations. Overkill for my use but fun to implement:
function relativeDuration(duration: Duration, opts?: Opts): string {
    let sigU = opts?.significantUnits ?? 2
    if (sigU < 1) {
        throw Error("Signficant units can't be < 1")
    }

    let units = opts?.units ?? defaultUnits
    // Make sure units are ordered in descending significance:
    units = allUnits.filter(it => units.includes(it))

    
    let negative = duration.valueOf() < 0
    if (negative) { duration = duration.negate() }
    duration = duration.shiftTo(...units)

    // Remove unnecessary most-significant units:
    while (units.length > 1) {
        if (duration.get(units[0]) > 0) {
            // we've found the most significant unit:
            break
        }

        units = units.slice(1)
        duration = duration.shiftTo(...units)
    }

    units = units.slice(0, sigU)
    duration = duration.shiftTo(...units)
    // If the last unit has fractional bits, we don't care. We're explicitly limiting significant units:
    let lastUnit = units[units.length - 1]
    duration = duration.set({
        [lastUnit]: Math.floor(duration.get(lastUnit))
    })

    let relative = duration.toHuman()
    if (negative) {
        return `${relative} ago`
    } else {
        return `in ${relative}`
    }
}

interface Opts {
    // Default: 2
    significantUnits?: number

    // Default: all but quarters & months
    units?: (keyof DurationObjectUnits)[]
}

const allUnits: ReadonlyArray<keyof DurationObjectUnits> = ["years", "quarters", "months", "weeks", "days", "hours", "minutes", "seconds", "milliseconds"]
// No quarters/weeks:
const defaultUnits: ReadonlyArray<keyof DurationObjectUnits> = ["years", "months", "days", "hours", "minutes", "seconds", "milliseconds"]
