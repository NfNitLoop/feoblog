<div class="item">
<div class="body">

    <h1>Sync</h1>
    <p>Synchronize your posts (and your feed) from/to multiple servers</p>
</div>
</div>


<div class="item">
<div class="body">
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
</div>

<div class="item">
<div class="body">
        <h1>Publish My Posts</h1>
    <p>Copy your posts (and profile updates, etc.) from this server to all servers listed in your profile</p>

    <Button on:click={publishMyPosts}>Sync</Button>

    <TaskTrackerView bind:tracker={publishMyPostsTracker}/>
</div>
</div>


<script language="ts">
import type { Task } from "svelte/internal";

import type { Writable } from "svelte/store"
import type { Item, ItemListEntry, Profile } from "../../protos/feoblog";
import type { AppState } from "../../ts/app"
import { Client, Signature, UserID } from "../../ts/client"
import { readableSize, TaskTracker, validateServerURL } from "../../ts/common";
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

let syncMyFeedTracker = new TaskTracker()



function syncMyFeed() {
    syncMyFeedTracker.run("Syncing my feed", syncMyFeedTask)
}

async function syncMyFeedTask(tracker: TaskTracker) {
    let local = $appState.client

    // If there's a single source server provided, only sync from that.
    let sourceServer = serverURL

    let myProfileItem: Item|undefined
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
        myProfileItem = result.item
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
            if (!myProfileItem || result.item.timestamp_ms_utc > myProfileItem.timestamp_ms_utc) {
                myProfile = result.item.profile
                $appState.userProfileChanged()
            }
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

    // Now fill in any file attachments for the sync:
    await tracker.runSubtask("Syncing file attachments", async (tracker) => {
        await syncFeedAttachments({
            sourceServer,
            tracker,
            to: local,
            userID,
            profile: myProfile
        })
    })
}

type SyncFeedAttachmentsArgs = {
    // An optional single server from whcih we're syncing data. (Else: use profile data.)
    sourceServer?: string|undefined,

    tracker: TaskTracker,

    // The destination we want to sync files to:
    to: Client,

    // UserID of the user's feed we're syncing.
    userID: UserID,
    // Profile of the user's feed we're syncing.
    profile: Profile|null|undefined,

}

// For each profile that a user follows (including itself),
// check their items for missing attachments and attempt to sync them.
// It may seem a bit inefficient to make a pass re-reading the Items from the server,
// but this method is more resilient to failures part-way through syncing files.
// i.e.: the next sync will still find that the file is missing.
// returns: Number of bytes of files copied.
async function syncFeedAttachments({sourceServer, tracker, to, userID, profile}: SyncFeedAttachmentsArgs): Promise<void> {

    let bytesCopied = 0

    let myServers = new Set<string>()

    if (sourceServer) {
        myServers.add(sourceServer)
    } else if (profile) {
        myServers = serversFromProfile(profile, tracker)
    }

    if (myServers.size === 0) {
        tracker.warn("No servers specified for current user. Can't sync user's files.")   
    } else {
        bytesCopied += await tracker.runSubtask(`User ${userID} ("${profile?.display_name}")`, async (tracker) => {
            return syncUserAttachments({tracker, fromServers: myServers, to, userID})
        })
    }



    let follows = profile?.follows || []
    for (let follow of follows) {
        let uid = UserID.fromBytes(follow.user.bytes)

        let followServers = myServers
        if (!sourceServer) {
            let result = await to.getProfile(uid)
            if (result) {
                // Check our own servers first, to lessen load on others.
                // Also handles the case when our follows for some reason didn't specify a server.
                followServers = union(myServers, serversFromProfile(result.item.profile))
            }
        }


        try {
            await tracker.runSubtask(`User ${uid} ("${follow.display_name}")`, (tracker) => {
                return syncUserAttachments({tracker, fromServers: followServers, to, userID: uid})
            })
        } catch (_ignored) {
            // The tracker will have logged and reported the exception.
        }
    }
    

    

    tracker.log(`Copied ${readableSize(bytesCopied)}`)
}

type SyncUserAttachmentsArgs = {
    tracker: TaskTracker,
    userID: UserID,
    fromServers: Set<string>,
    to: Client,
}

async function syncUserAttachments({tracker, userID, fromServers, to}: SyncUserAttachmentsArgs): Promise<number> {
    let bytesCopied = 0

    if (fromServers.size == 0) {
        tracker.warn(`No servers to sync for this user`)
        return bytesCopied
    }

    // We can probably speed this along by prefetching Items, but it seems pretty zippy already.
    for await (let entry of to.getUserItems(userID)) {
        let info = entryInfoFrom(entry, userID)

        // For now, only posts have attachments:
        if (info.type && (info.type != "post")) {
            continue
        }

        // Skipping check to speed things along. If the server lies to you about attachments, then...
        // we'll try to send more attachments to it? Or we won't? I don't know if I should be worried
        // about this case.
        let item = await to.getItem(info.userID, info.signature, {skipSignatureCheck: true})
        if (!item) {
            // Server told us this item was there!? 
            throw `Error fetching item ${info.signature}`
        }

        let attachments = getAttachments(item)
        for (let attachment of attachments) {
            bytesCopied += await syncAttachment({
                userID, 
                signature: info.signature,
                fileName: attachment.name,
                fromServers,
                to,
                tracker
            })
        }
        


    }

    return bytesCopied
}

type SyncAttachmentParams = {
    userID: UserID,
    signature: Signature,
    fileName: string,
    to: Client,
    fromServers: Set<string>,
    tracker: TaskTracker
}

async function syncAttachment({userID, signature, fileName, to, fromServers, tracker}: SyncAttachmentParams): Promise<number> {
    let bytesCopied = 0

    let targetMeta = await to.headAttachment(userID, signature, fileName)
    if (targetMeta.exists) {
        // No need to log anything here, this will be the common case:
        return bytesCopied
    }

    return tracker.runSubtask(`Syncing ${fileName}`, async (tracker) => {
        tracker.log(`For item ID: ${signature}`)
        if (targetMeta.exceedsQuota) {
            tracker.warn(`Copying would exceed the user's quota.`)
            return bytesCopied
        }

        let buffer: ArrayBuffer|null = null
        for (let server of fromServers) {
            let from = new Client({base_url: server})

            buffer = await tracker.runSubtask(`Downloading from ${from.url}`, async (tracker) => {
                let buffer = await from.getAttachment(userID, signature, fileName)
                if (!buffer) { tracker.log("Not found") }
                return buffer
            });
            if (buffer) break
        }

        if (!buffer) {
            tracker.warn("Could not find file on any known server.")
            return bytesCopied
        }

        try {
            await tracker.runSubtask(`Uploading to ${to.url}`, async (tracker) => {
                let blob = new Blob([buffer!])
                let response = await to.putAttachment(userID, signature, fileName, blob)
                tracker.log(`Success. ${response.status}: ${response.statusText}`)
            })
        } catch (_ignored) {
            // The subtask will have recorded the error.
            return bytesCopied
        }

        bytesCopied += buffer.byteLength
        tracker.log(`Copied ${readableSize(bytesCopied)}`)
        
        return bytesCopied
    })
}

function getAttachments(item: Item): AttachmentInfo[] {
    let infos: AttachmentInfo[] = []

    let attachments = item?.post?.attachments?.file || []

    for (let attachment of attachments) {
        infos.push({
            name: attachment.name,
            size: attachment.size,
        })
    }

    return infos
}

type AttachmentInfo = {
    name: string,
    size: number,
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
    // let localSignatures = await loadAllSignatures(local, userID)
    let localItems = await loadItemEntries(local, userID);

    for (let server of servers) {
        try {
            let syncCount = 0
            let errorSaving = false

            await tracker.runSubtask(`Syncing from ${server}`, async (tracker) => {
                let remote = new Client({base_url: server})

                for await (let listEntry of remote.getUserItems(userID)) {
                    let info
                    let signature 
                    try {
                        info = entryInfoFrom(listEntry, userID);
                        signature = info.signature
                    } catch (e) {
                        tracker.error(`Invalid signature from server: ${listEntry.signature.bytes}`)
                        continue
                    }
                    if (localItems.has(signature.toString())) continue

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

                    localItems.set(signature.toString(), info)
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

async function loadItemEntries(client: Client, userID: UserID): Promise<Map<string, EntryInfo>> {
    // Note: must use string keys so JS == works properly.
    let entries = new Map<string, EntryInfo>()
    for await (let entry of client.getUserItems(userID)) {
        let info = entryInfoFrom(entry, userID)
        entries.set(info.signature.toString(), info)
    }
    return entries
}

function entryInfoFrom(entry: ItemListEntry, userID: UserID): EntryInfo {
    let sig = Signature.fromBytes(entry.signature.bytes)

    return {
        userID,
        signature: sig,
        timestamp: entry.timestamp_ms_utc,
        type: entryTypeFromID(entry.item_type)
    }
}

type ItemEntryType = "comment"|"post"|"profile"|undefined

// Work around https://github.com/thesayyn/protoc-gen-ts/issues/32
// TODO: Consider switching Protobuf libraries.
function entryTypeFromID(typeID: any): "comment"|"post"|"profile"|undefined {
    switch(typeID) {
        case 1: return "post"
        case 2: return "profile"
        case 3: return "comment"
    }
    return undefined
}

// Information from a Protobuf ItemEntry
type EntryInfo = {
    userID: UserID,
    signature: Signature,
    timestamp: number,
    type?: ItemEntryType,
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
    let bytes = await from.getItemBytes(userID, signature, {skipSignatureCheck: true})
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
}





// Return only valid servers from a Profile.
function serversFromProfile(profile: Profile|undefined|null, tracker = new TaskTracker()): Set<string> {
    if (!profile) return new Set()
    
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
    let localItems = await loadItemEntries(local, userID)

    for (let server of servers) {
        try {
            let remote = new Client({base_url: server})
            // Loading full list because I'm lazy. We could order local/remote items and then iterate.
            let remoteSigs = await loadItemEntries(remote, userID)
            for (let [sig, entry] of localItems) {
                if (remoteSigs.has(sig)) continue
                await syncUserItem({
                    userID,
                    signature: entry.signature,
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

    // TODO: Sync files.
}


</script>