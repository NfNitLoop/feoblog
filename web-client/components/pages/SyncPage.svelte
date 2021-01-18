<div class="item">
    <h1>Sync</h1>
    <p>Synchronize your posts (and your feed) from/to multiple servers</p>
</div>

<div class="item">
    <h1>Bootstrap Profiles</h1>

    <p>If you're moving to this server from another, the first step is to copy your profile, and the profiles of those you follow.
        Each profile contains a list of servers where that userID's posts can be found, and will be used for a full sync.
    </p>

    <InputBox 
        label="Server URL"
        placeholder="https://feoblog.example.com"
        validationCallback={checkServerURL}
        disabled={$profileTask.isRunning}
        bind:errorMessage={serverURLError}
        bind:value={serverURL}
    />
    <UserIdInput
        label="User ID"
        placeholder="Default: current user's ID"
        disabled={$profileTask.isRunning}
        bind:valid={bootstrapUserIDValid}
        bind:value={bootstrapUserID}
    />
    <Button
        disabled={!haveValidServerURL || (bootstrapUserID.length > 0 && !bootstrapUserIDValid) || $profileTask.isRunning}
        on:click={bootstrapProfiles}
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
import type { Profile } from "../../protos/feoblog";

import type { AppState } from "../../ts/app"
import { Client, UserID } from "../../ts/client"
import Button from "../Button.svelte"
import InputBox from "../InputBox.svelte"
import UserIdInput from "../UserIDInput.svelte";

export let appState: Writable<AppState>

let userID: UserID
$: userID = function() {
    let id = $appState.loggedInUser
    if (!id) throw `Must be logged in`
    return id
}()


const serverURLPattern = /^(https?:\/\/[^/]+)\/?$/
let serverURL = ""
let serverURLError = ""
$: haveValidServerURL = (serverURL != "") && serverURLError === ""

let bootstrapUserID = ""
let bootstrapUserIDValid = false

function checkServerURL(url: string): string {
    if (url === "") {
        return "" // Don't show error in the empty case.
    }

    let match = serverURLPattern.exec(url)
    if (match === null) {
        return "Invalid URL format"
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

function bootstrapProfiles() {
    let tracker = new TaskTracker()
    tracker.store = profileTask

    tracker.run(() => bootstrapProfilesTask(tracker))
}

async function bootstrapProfilesTask(tracker: TaskTracker) {
    
    let local = $appState.client

    let match = serverURLPattern.exec(serverURL)
    if (!match) throw "Invalid profile URL?"
    let remoteURL = match[1]
    let remote = new Client({
        base_url: remoteURL
    })

    let uid = userID // default
    if (bootstrapUserID) {
        uid = UserID.fromString(bootstrapUserID)
    }
    
    tracker.log(`Syncing from ${serverURL}`)
    tracker.log(`Updating profile ${uid}`)
    let profile = await syncOneProfile(tracker, local, remote, uid)

    if (profile.follows.length == 0) return

    tracker.log("Syncing followed profiles")
    // Convert UIDs up front, and error quickly if there's an invalid one:
    let follows = profile.follows.map((f) => {
        return {
            name: f.display_name,
            userID: UserID.fromBytes(f.user.bytes),
        }
    })

    // This could be done in parallel, logging will need to handle that better.
    for (let {name, userID} of follows) {
        tracker.log(`Syncing follow "${name}" (${userID})`)
        try {
            await syncOneProfile(tracker, local, remote, userID)
        } catch (e) {
            console.error("fetching profile", e)
            tracker.error(`${e}`)
        }
    }
}

// Syncs one profile from remote to local.
// Throws an exception if there was an error.
// returns the newest Profile involved in the sync.
async function syncOneProfile(tracker: TaskTracker, local: Client, remote: Client, userID: UserID): Promise<Profile> {

    // Make requests in parallel:
    let remoteRequest = remote.getProfile(userID)
    let localRequest = local.getProfile(userID)

    let remoteProfile = await remoteRequest
    if (remoteProfile == null) throw `Can't find remote profile for ${userID}`

    let localProfile = await localRequest

    if (localProfile !== null) {
        if (localProfile.signature.toString() == remoteProfile.signature.toString()) {
            tracker.log("Profiles are identical")
            return localProfile.item.profile
        }
         
        if (localProfile.item.timestamp_ms_utc >= remoteProfile.item.timestamp_ms_utc) {
            tracker.log("Remote profile is not newer than local profile.")
            return localProfile.item.profile
        }
    }

    tracker.log("Saving profile locally")
    await local.putItem(userID, remoteProfile.signature, remoteProfile.bytes)
    tracker.log("Saved")

    return remoteProfile.item.profile
}

</script>