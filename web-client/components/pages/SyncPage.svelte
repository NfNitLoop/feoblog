<div class="item">
    <h1>Sync</h1>
    <p>Synchronize your posts (and your feed) from/to multiple servers</p>
</div>


<div class="item">
    <h1>Sync My Feed</h1>
    <p>Copies your own posts, and posts of those you follow, from any remote servers to this one.</p>

    <InputBox 
        label="Server URL"
        placeholder="(Optional. Default: Servers listed in profiles)"
        validationCallback={validateServerURL}
        disabled={syncMyFeedTracker.isRunning}
        bind:errorMessage={serverURLError}
        bind:value={serverURL}
    />
    <Button 
        on:click={syncMyFeed} 
        disabled={syncMyFeedTracker.isRunning || serverURLError != ""}
    >Sync</Button>

    <TaskTrackerView bind:tracker={syncMyFeedTracker}/>
</div>

<div class="item">
    <h1>Publish My Posts</h1>
    <p>Copy your posts (and profile updates, etc.) from this server to all servers listed in your profile</p>

    <Button on:click={publishMyPosts}>Sync</Button>

    <TaskTrackerView bind:tracker={publishMyPostsTracker}/>
</div>


<script language="ts">
import type { Writable } from "svelte/store"
import type { Profile } from "../../protos/feoblog";

import type { AppState } from "../../ts/app"
import { Client, Signature, UserID } from "../../ts/client"
import { TaskTracker, validateServerURL } from "../../ts/common";
import Button from "../Button.svelte"
import InputBox from "../InputBox.svelte"
import TaskTrackerView from "../TaskTrackerView.svelte";

export let appState: Writable<AppState>

let userID: UserID
$: userID = function() {
    let id = $appState.loggedInUser
    if (!id) throw `Must be logged in`
    return id
}()


let serverURL = ""
let serverURLError = ""

let bootstrapUserID = ""

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

    // If there's a single source server provided, only sync from that.
    let sourceServer = serverURL

    let myProfile: Profile|undefined
    let myServers = new Set<string>()

    if (sourceServer) {
        myServers.add(sourceServer)
    } else {
        let result = await local.getProfile(userID)
        if (!result) {
            tracker.error("Current user has no profile, and no server specified. Can't sync anything.")
            return
        }
        myProfile = result.item.profile
        myServers = serversFromProfile(myProfile, tracker)
     }

    if (myServers.size === 0) {
        tracker.warn("No servers specified for current user. Can't sync current user's items.")   
    } else {
        await tracker.runSubtask("Current user's items", (tracker) => {
            return syncUserItems({tracker, local, userID, servers: myServers})
        })

        // Re-load current user's profile, which may have been updated as a result of the sync
        let result = await local.getProfile(userID)
        if (result) {
            myProfile = result.item.profile
            if (!sourceServer) {
                myServers = serversFromProfile(myProfile)
            }
        }
    }

    await tracker.runSubtask("Follows' items", async (tracker) => {
        if (!myProfile) {
            tracker.warn("User has no profile. No follows to sync.")
            return
        }

        for (let follow of myProfile.follows) {
            // Sync items from each of my follows.
            try {
                let uid = UserID.fromBytes(follow.user.bytes)

                let followServers = myServers
                if (!sourceServer) {
                    let result = await local.getProfile(uid)
                    if (result) {
                        // Check our own servers first, to lessen load on others.
                        // Also handles the case when our follows for some reason didn't specify a server.
                        followServers = union(myServers, serversFromProfile(result.item.profile))
                    }
                }

                await tracker.runSubtask(`Items for ${uid} ("${follow.display_name}")`, (tracker) => {
                    return syncUserItems({tracker, local, userID: uid, servers: followServers})
                })
            } catch (e) {
                // Syncing one user's items shouldn't fail others:
                tracker.error(e)
            }
        }
    })

}

function union<T>(...sets: Set<T>[]): Set<T> {
    let out = new Set<T>()
    for (let s of sets) {
        s.forEach((item) => out.add(item))
    }
    return out
}

type SyncOptions = {
    tracker: TaskTracker
    local: Client
    userID: UserID

    // Remote servers to sync from
    servers: Set<string>
}

async function syncUserItems({tracker, local, userID, servers}: SyncOptions): Promise<void> {

    if (servers.size === 0) {
        tracker.warn(`No servers found to sync ${userID}`)
        return
    }

    // TODO: Put some optional limit on how far back we sync. 
    let localSignatures = await loadAllSignatures(local, userID)

    for (let server of servers) {
        try {
            let syncCount = 0
            let errorSaving = false

            await tracker.runSubtask(`Syncing from ${server}`, async (tracker) => {
                let remote = new Client({base_url: server})

                for await (let listEntry of remote.getUserItems(userID)) {
                    let signature 
                    try {
                        signature = Signature.fromBytes(listEntry.signature.bytes)
                    } catch (e) {
                        tracker.error(`Invalid signature from server: ${listEntry.signature.bytes}`)
                        continue
                    }
                    if (localSignatures.has(signature.toString())) continue

                    try {
                        await syncUserItem({
                            userID,
                            signature,
                            to: local,
                            from: remote,
                            tracker,
                        })
                        ++syncCount
                    } catch (e) {
                        tracker.error(`Error saving item: ${e}`)
                        tracker.warn("This may mean that the user can not post to the server, or has exceeded their quota. Skipping")
                        errorSaving = true
                        return // out of the subtask.
                    }

                    localSignatures.add(signature.toString())
                } //for
            }) // subTask

            tracker.log(`Copied ${syncCount} new items`)
            if (errorSaving) {
                // Very likely can't save more items for this user. Don't try more.
                return
            }
        } catch (e) {
            // One server failing shouldn't stop us from syncing from others.
            tracker.error(`${e}`)
            tracker.warn("Skipping this server")
        }

    }
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