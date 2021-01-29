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
        disabled={profileTask.isRunning}
        bind:errorMessage={serverURLError}
        bind:value={serverURL}
    />
    <UserIdInput
        label="User ID"
        placeholder="Default: current user's ID"
        disabled={profileTask.isRunning}
        bind:valid={bootstrapUserIDValid}
        bind:value={bootstrapUserID}
    />
    <Button
        disabled={!haveValidServerURL || (bootstrapUserID.length > 0 && !bootstrapUserIDValid) || profileTask.isRunning}
        on:click={bootstrapProfiles}
    >Sync</Button>

    <TaskTrackerView bind:tracker={profileTask}/>
</div>

<div class="item">
    <h1>Sync My Feed</h1>
    <p>Copies your own posts, and posts of those you follow, from any remote servers (listed in profiles) to this one.</p>
    <Button on:click={syncMyFeed} disabled={syncMyFeedTracker.isRunning}>Sync</Button>

    <TaskTrackerView bind:tracker={syncMyFeedTracker}/>
</div>

<div class="item">
    <h1>Publish My Posts</h1>
    <p>Copy your posts (and profile updates, etc.) from this server to all servers listed in your profile</p>

    <Button on:click={publishMyPosts}>Sync</Button>

    <TaskTrackerView bind:tracker={publishMyPostsTracker}/>
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


let profileTask = new TaskTracker()
let syncMyFeedTracker = new TaskTracker()

function bootstrapProfiles() {
    profileTask.run("Copying profiles", bootstrapProfilesTask)
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
    syncMyFeedTracker.run("Syncing my feed", syncMyFeedTask)
}

async function syncMyFeedTask(tracker: TaskTracker) {
    let local = $appState.client
    
    let profile = await tracker.runSubtask("Syncing current user's items", (subTracker) => {
        return syncUserItems({tracker: subTracker, local, userID})
    })

    let myServers = serversFromProfile(profile)

    await tracker.runSubtask("Syncing follows' items", async (subtask) => {
        for (let follow of profile.follows) {
            // Sync items from each of my follows.
            try {
                let uid = UserID.fromBytes(follow.user.bytes)
                await subtask.runSubtask(`Syncing items for ${uid} "${follow.display_name}"`, (t) => {
                    return syncUserItems({tracker: t, local, userID: uid, extraServers: myServers})
                })
            } catch (e) {
                // Syncing one user's items shouldn't fail others:
                tracker.error(e)
            }
        }
    })

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

async function syncUserItems({tracker, local, userID, extraServers}: SyncOptions): Promise<Profile> {
    let response = await local.getProfile(userID)

    // TODO: Hmm, I suppose now that we take `extraServers` we could just try to sync from those.
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
    let localSignatures = await loadAllSignatures(local, userID)

    for (let server of servers) {
        try {
            tracker.log(`Syncing from ${server}`)
            let remote = new Client({base_url: server})

            for await (let listEntry of remote.getUserItems(userID)) {
                let signature = Signature.fromBytes(listEntry.signature.bytes)
                if (localSignatures.has(signature.toString())) continue
                syncUserItem({
                    userID,
                    signature,
                    to: local,
                    from: remote,
                    tracker,
                })

                localSignatures.add(signature.toString())
            }
        } catch (e) {
            // One server failing shouldn't stop us from syncing from others.
            tracker.error(`${e}`)
            tracker.warn("Skipping this server")
        }

    }

    return profile
}

// TODO: In the future, we can do some more efficient loading. Since getUserItems() returns 
// items in tiemstamp order, we can do a merge-ish comparison as we walk the local/remote lists to
// more efficiently diff them.
//
// For now, we'll load the IDs of all known items and just use the Set(s) to diff.
async function loadAllSignatures(client: Client, userID: UserID): Promise<Set<string>> {
    // Note, we'll use a string because objects use identity comparisons, not equality.
    let sigs = new Set<string>()

    for await (let listEntry of client.getUserItems(userID)) {
        let sig = Signature.fromBytes(listEntry.signature.bytes)
        sigs.add(sig.toString())
    }
    return sigs
}

type SyncUserItemParams = {
    to: Client
    from: Client
    tracker: TaskTracker
    userID: UserID
    signature: Signature
}

async function syncUserItem({userID, signature, to, from, tracker}: SyncUserItemParams) {

    tracker.log(`Copying ${signature}`)
    let bytes = await from.getItemBytes(userID, signature)
    if (!bytes) {
        // This would be weird, since the remote server just listed the item for us.
        // But I guess it shouldn't block syncing further items?
        tracker.warn("404 (not found) from `from` server")
        return
    }

    // Throws & exits if we couldn't put an item.
    // One possible reason is that the user is not "known" to the server, so the server
    // won't hold items for that user.  Another is that the user has reached their quota.
    await to.putItem(userID, signature, bytes)

    // TODO: Once implemented, also sync file contents.
}

// Return onl valid servers from a Profile.
// Optionally tracker.warn() about broken ones.
function serversFromProfile(profile: Profile, tracker = new TaskTracker()): Set<string> {
    return new Set(
        profile.servers.map(s => s.url).filter(url => {
            let error = validateServerURL(url)
            if (error) {
                tracker.warn(`Skipping invalid server URL. ${error}: ${url}`)
                return false
            }
            return true
        })
    )
}

let publishMyPostsTracker = new TaskTracker()

function publishMyPosts() {
    publishMyPostsTracker.run("Publish my posts", publishMyPostsTask)
}

async function publishMyPostsTask(tracker: TaskTracker) {
    let local = $appState.client
    let result = await local.getProfile(userID)
    if (!result) throw `Current user does not have a local profile.`

    let profile = result.item.profile
    let servers = serversFromProfile(profile)
    if (servers.size === 0) {
        throw `User profile doesn't specify any servers.`
    }

    // Loading full list once, because we may need it N times below:
    // TODO: as above, we could limit this to some shorter timespan by default.
    let localSigs = await loadAllSignatures(local, userID)

    for (let server of servers) {
        try {
            let remote = new Client({base_url: server})
            // Loading full list because I'm lazy. If we had timestamps in localSigs, we could iterate:
            let remoteSigs = await loadAllSignatures(remote, userID)
            for (let sig of localSigs) {
                if (remoteSigs.has(sig)) continue
                await syncUserItem({
                    userID,
                    signature: Signature.fromString(sig),
                    from: local,
                    to: remote,
                    tracker
                })
            }

        } catch (e) {
            tracker.error(`${e}`)
            tracker.warn("Skipping this server")
        }
    }
}


</script>