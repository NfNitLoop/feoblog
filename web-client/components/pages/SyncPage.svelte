<div class="item">
    <h1>Sync</h1>
    <p>Synchronize your posts (and your feed) from/to multiple servers</p>
</div>

<div class="item">
    <h1>One profile</h1>

    <p>Synchronize one profile to this server. Useful if you or someone you follow does not yet have a profile saved on this server.</p>

    <InputBox 
        label="Remote Profile URL"
        placeholder="https://feoblog.example.com/u/<userID>"
        validationCallback={checkProfileURL}
        bind:errorMessage={profileURLError}
        bind:value={profileURL}
    />
    <Button
        disabled={profileURLError.length > 0 || profileURL.length == 0 || $profileTask.isRunning}
        on:click={syncOneProfile}
    >Sync</Button>

    {#if $profileTask.logs.length > 0}
    <ul>
        {#each $profileTask.logs as entry}
            {#if entry.isError}
                <li class=error>{entry.message}</li>
            {:else if entry.isWarning}
                <li>⚠ {entry.message}</li>
            {:else}
                <li>{entry.message}</li>
            {/if}
        {/each}
    </ul>
    {/if}
</div>


<script language="ts">
import { DateTime } from "luxon";
import type { Writable } from "svelte/store"
import { writable } from "svelte/store"

import type { AppState } from "../../ts/app"
import { Client, UserID } from "../../ts/client"
import Button from "../Button.svelte"
import InputBox from "../InputBox.svelte"

export let appState: Writable<AppState>

let userID: UserID
$: userID = function() {
    let id = $appState.loggedInUser
    if (!id) throw `Must be logged in`
    return id
}()

let profileURL = ""
let profileURLError = ""

const profileURLPattern = /^(https?:\/\/[^/]+)\/u\/([^/]+)\/?$/
function checkProfileURL(url: string): string {
    if (url === "") {
        return "" // Don't show error in the empty case.
    }

    let match = profileURLPattern.exec(url)
    if (match === null) {
        return "Invalid URL format"
    }

    if (match.length != 3) {
        return `Expected 3 matches, found ${match.length}`
    }

    try {
        let id = match[2]
        UserID.fromString(id)
    } catch (e) {
        return `Invalid userID: ${e}`
    }

    return ""
}


// Tracks the progress of some long-running async task.
class TaskTracker 
{
    // A store that will geet updated every time this object changes
    store: Writable<TaskTracker>|null = null
 
    _isRunning = false
    get isRunning() { return this._isRunning }

    _logs: LogEntry[] = []
    get logs(): ReadonlyArray<LogEntry> {
        return this._logs
    }

    async run(asyncTask: () => Promise<void>): Promise<void> {
        this.clear()
        this._isRunning = true
        this.log("Begin") // calls notify()
        try {
            await asyncTask()
        } catch (e) {
            this.error(`Task threw an exception: ${e}`)
        }
        this._isRunning = false
        this.log("Done") // calls notify()
    }

    private notify() {
        if (this.store) this.store.set(this)
    }

    clear() {
        this._logs = []
        this.notify()
    }

    private writeLog(log: LogEntry) {
        this._logs.push(log)
        this.notify()
    }

    error(message: string) {
        this.writeLog({
            message,
            isError: true,
            timestamp: DateTime.local().valueOf()
        })
    }

    log(message: string) {
        this.writeLog({
            message,
            timestamp: DateTime.local().valueOf()
        })
    }

    warn(message: string) {
        this.writeLog({
            message,
            isWarning: true,
            timestamp: DateTime.local().valueOf()
        })
    }
}


type LogEntry = {
    timestamp: number
    message: string
    isError?: boolean
    isWarning?: boolean
}

let profileTask = writable(new TaskTracker())

function syncOneProfile() {
    let tracker = new TaskTracker()
    tracker.store = profileTask

    tracker.run(() => syncOneProfileTask(tracker))
}

async function syncOneProfileTask(tracker: TaskTracker) {
    
// let userProfile: Promise<Profile|null> 
// $: userProfile = loadUserProfile(userID)
// async function loadUserProfile(userID: UserID): Promise<Profile|null> {
//     // Note: NOT necessarily the latest profile, we'll do exhaustive searches
//     // as part of sync.
//     let result = await $appState.client.getProfile(userID)
//     if (!result) return null
//     return result.item.profile
// }
    let match = profileURLPattern.exec(profileURL)
    if (!match) throw "Invalid profile URL?"

    let local = $appState.client
    let remote = new Client({
        base_url: match[1]
    })

    let userID = UserID.fromString(match[2])
    tracker.log(`Updating profile from ${profileURL}`)

    // Make requests in parallel:
    let remoteRequest = remote.getProfile(userID)
    let localRequest = local.getProfile(userID)

    let remoteProfile = await remoteRequest
    if (remoteProfile == null) throw `Can't find profile for ${profileURL}`
    tracker.log(`Fetched remote profile`)

    let localProfile = await localRequest
    tracker.log(`Fetched local profile`)

    if (localProfile !== null) {
        if (localProfile.signature.toString() == remoteProfile.signature.toString()) {
            tracker.log("Profiles are identical")
            return
        }
         
        if (localProfile.item.timestamp_ms_utc >= remoteProfile.item.timestamp_ms_utc) {
            tracker.log("Remote profile is not newer than local profile.")
            return
        }
    }

    tracker.log("Saving profile locally")
    await local.putItem(userID, remoteProfile.signature, remoteProfile.bytes)
    tracker.log("Saved")
}

</script>