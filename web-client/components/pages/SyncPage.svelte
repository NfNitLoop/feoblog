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
        validationCallback={validateServerURL}
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

    <TaskTrackerView tracker={profileTask}/>
</div>

<div class="item">
    <h1>Sync My Feed</h1>
    <p>Copies your own posts, and posts of those you follow, from any remote servers (listed in profiles) to this one.</p>
    <Button on:click={syncMyFeed}>Sync</Button>

    <TaskTrackerView tracker={syncMyFeedTracker}/>
</div>

<div class="item">
    <h1>Publish my posts</h1>
    <p>Copy your posts (and profile updates, etc.) from this server to all servers listed in your profile</p>
</div>


<script language="ts">
import { DateTime } from "luxon";
import type { Writable } from "svelte/store"
import { writable } from "svelte/store"
import type { ItemListEntry, Profile } from "../../protos/feoblog";

import type { AppState } from "../../ts/app"
import { Client, Signature, UserID } from "../../ts/client"
import { TaskTracker, validateServerURL } from "../../ts/common";
import Button from "../Button.svelte"
import InputBox from "../InputBox.svelte"
import TaskTrackerView from "../TaskTrackerView.svelte";
import UserIdInput from "../UserIDInput.svelte";

export let appState: Writable<AppState>

let userID: UserID
$: userID = function() {
    let id = $appState.loggedInUser
    if (!id) throw `Must be logged in`
    return id
}()


let serverURL = ""
let serverURLError = ""
$: haveValidServerURL = (serverURL != "") && serverURLError === ""

let bootstrapUserID = ""
let bootstrapUserIDValid = false





let profileTask = writable(new TaskTracker())
let syncMyFeedTracker = writable(new TaskTracker())

function bootstrapProfiles() {
    let tracker = new TaskTracker()
    tracker.store = profileTask

    tracker.run(() => bootstrapProfilesTask(tracker))
}

async function bootstrapProfilesTask(tracker: TaskTracker) {
    
    let local = $appState.client

    if (validateServerURL(serverURL)) throw `Invalid server URL`
    let remoteURL = serverURL
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

function syncMyFeed() {
    let tracker = new TaskTracker()
    tracker.store = syncMyFeedTracker

    tracker.run(() => syncMyFeedTask(tracker))

}

async function syncMyFeedTask(tracker: TaskTracker) {
    let local = $appState.client
    
    let profile = await syncItems({tracker, local, userID})
    let myServers = serversFromProfile(profile)

    tracker.log("Syncing follows' items")
    for (let follow of profile.follows) {
        // Sync items from each of my follows.
        try {
            let uid = UserID.fromBytes(follow.user.bytes)
            await syncItems({tracker, local, userID: uid, extraServers: myServers})
        } catch (e) {
            tracker.error(e)
        }
    }
}

type SyncOptions = {
    tracker: TaskTracker
    local: Client
    userID: UserID

    // Additional servers to fetch from.
    // Helps when your friends forgot to specify their server, but you're homed there.
    extraServers?: Set<string>

    // If true, `extraServers` completely replaces (instead of extends) the list provided by `userID`'s profile
    serversOverride?: boolean
}

async function syncItems({tracker, local, userID, extraServers}: SyncOptions): Promise<Profile> {
    tracker.log(`Syncing items for ${userID}`)
    let response = await local.getProfile(userID)
    if (!response) throw `${userID} has no profile`
    let profile = response.item.profile

    // Use a set to avoid hitting the same server multiple times:
    let servers = serversFromProfile(profile, tracker)
    if (extraServers) {
        for (let s of extraServers) servers.add(s)
    }
    
    if (servers.size === 0) {
        tracker.log("User profile has 0 servers.")
        return profile
    }

    // TODO: Put some optional limit on how far back we sync. 
    // For now, we'll load the IDs of all known local items.
    // Note, we'll use a string because objects use identity comparisons, not equality.
    let localSignatures: Set<string> = new Set()
    for await (let listEntry of local.getUserItems(userID)) {
        let sig = Signature.fromBytes(listEntry.signature.bytes)
        localSignatures.add(sig.toString())
    }

    for (let server of servers) {
        tracker.log(`Syncing from ${server}`)
        let remote = new Client({base_url: server})

        for await (let listEntry of remote.getUserItems(userID)) {
            let sig = Signature.fromBytes(listEntry.signature.bytes)
            if (localSignatures.has(sig.toString())) continue // already here, no need to sync.

            tracker.log(`Copying ${sig}`)
            let bytes = await remote.getItemBytes(userID, sig)
            if (!bytes) {
                tracker.warn("404 (not found) from remote server")
                continue
            }
            await local.putItem(userID, sig, bytes)
            
            // If we sync more servers later, don't need to re-sync this:
            localSignatures.add(sig.toString())

            // TODO: Eventually, also sync file contents.
        }

    }

    return profile
}

function serversFromProfile(profile: Profile, tracker = new TaskTracker()) {
    return new Set(
        profile.servers.map(s => s.url).filter(url => {
            let error = validateServerURL(url)
            if (error) {
                tracker.warn(`${error}: ${url}`)
                return false
            }
            return true
        })
    )
}

</script>