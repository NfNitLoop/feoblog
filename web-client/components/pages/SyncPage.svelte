<div class="item">
<div class="body">

    <h1>Sync</h1>
    <p>Synchronize your posts (and your feed) from/to other servers</p>
    <ul>
        <li><strong>Pull My Feed</strong>: Copies posts of those you follow from their various servers onto this one.</li>
        <li><strong>Push My Posts</strong>: Copies posts you've made on this server to any servers you've listed in your profile.</li>
    </ul>
</div>
</div>


<div class="item">
<div class="body">
    <InputBox 
        label="Server URL"
        placeholder="(Optional. Default: Servers listed in profiles)"
        validationCallback={validateServerURL}
        disabled={taskTracker.isRunning}
        bind:errorMessage={serverURLError}
        bind:value={serverURL}
    />

    <div class="buttons">
        <Button 
            on:click={syncMyFeed} 
            disabled={taskTracker.isRunning || serverURLError != ""}
        >Pull My Feed</Button>

        <Button
            on:click={publishMyPosts}
            disabled={taskTracker.isRunning || serverURLError != ""}
        >Push My Posts</Button>
    </div>

    <TaskTrackerView bind:tracker={taskTracker}/>
</div>
</div>

<style>
.buttons { margin: 1rem 0rem; }
</style>

<script language="ts">
import type { Writable } from "svelte/store"
import type { Item, ItemListEntry, Profile } from "../../protos/feoblog";
import type { AppState } from "../../ts/app"
import type { AttachmentMeta } from "../../ts/client"
import { Client, Signature, UserID } from "../../ts/client"
import { prefetch, readableSize, TaskTracker, validateServerURL } from "../../ts/common";
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

let taskTracker = new TaskTracker()



function syncMyFeed() {
    taskTracker.run("Syncing my feed", syncMyFeedTask)
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

// When pulling attachments to this server, sync from possibly multiple `fromServers`.
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
                await to.putAttachment(userID, signature, fileName, blob)
                tracker.log(`Success.`)
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

// When syncing local attachments to remote servers, send the attachment to multiple servers.
// We can even do it simultaneously. Handy.
type SendAttachmentMultiParams = {
    userID: UserID,
    signature: Signature,
    fileName: string,
    from: Client,
    toServers: Set<string>,
    tracker: TaskTracker
}

async function sendAttachmentMulti({userID, signature, fileName, toServers, from, tracker}: SendAttachmentMultiParams): Promise<number> {
    let bytesCopied = 0

    let destServers: Client[] = []
    for (let base_url of toServers) {
        destServers.push(new Client({base_url}))
    }

    let heads: [Client, Promise<AttachmentMeta>][] = []
    for (let dest of destServers) {
        let meta = dest.headAttachment(userID, signature, fileName)
        heads.push([dest, meta])
    }

    let needTheAttachment: Client[] = []
    for (let [dest, metaPromise] of heads) {
        let meta
        try {
            meta = await metaPromise
        } catch (e) {
            tracker.error(`Error from ${dest.url} : ${e}`)
            continue
        }
        if (meta.exists) continue;
        if (meta.exceedsQuota) {
            tracker.warn(`Sending to ${dest.url} would exceed quota.`)
            continue
        }
        needTheAttachment.push(dest)
    }

    if (needTheAttachment.length == 0) {
        // Nobody needs this file, no point in loading it:
        return bytesCopied
    }

    return tracker.runSubtask(`Syncing ${fileName}`, async (tracker) => {
        tracker.log(`For Item ${signature}`)
        let bytesCopied = 0

        let bufPromise = tracker.runSubtask(`Loading ${fileName} from ${from.url}`, async (tracker) => {
            return from.getAttachment(userID, signature, fileName)
        })

        let buffer: ArrayBuffer|null
        try {
            buffer = await bufPromise
        } catch (e) {
            // Will already be tracked as an error by the subtask.
            // But without this file, we can't proceed:
            return bytesCopied
        }
    
        if (!buffer) {
            return bytesCopied // for the same reason
        }
        
        let blob = new Blob([buffer])
        let fileSize = buffer.byteLength

        let uploads: Promise<number>[] = []
        for (let client of needTheAttachment) {
            let task = tracker.runSubtask(`Sending to ${client.url}`, async (tracker) => {
                await client.putAttachment(userID, signature, fileName, blob)
                tracker.log(`Sent ${readableSize(fileSize)}`)
                return fileSize
            })
            uploads.push(task)
        }

        for (let upload of uploads) {
            try {
                bytesCopied += await upload
            } catch (e) {
                // Already logged by the tracker.
            }
        }

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

    let syncCount = 0

    await tracker.runSubtask(`Syncing from ${[...servers].join(", ")}`, async (tracker) => {
        let itemsToSync = missingItems({tracker, local, userID, servers})

        let hadError = false

        let results = prefetch(itemsToSync, 4, async (args) => {
            try {
                await syncUserItem(args)
            } catch (e) {
                // TODO: This could be an error due to the remote being unable to serve the item,
                // or the local server being unable to accept it.  Distinguish and handle errors separately.
                tracker.error(`Error saving item: ${e}`)
                tracker.warn("This may mean that the user can not post to the server, or has exceeded their quota.")
                hadError = true
            }
        });

        for await (let result of results) {
            syncCount++
            if (hadError) break
        }
    })

    tracker.log(`Copied ${syncCount} new items`)
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

// Can throw:
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
    // Skipping client-side signature verification because we expect the accepting server
    // to validate it for us anyway.
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

// Get a list of items that exist on remote servers but not the local one.

async function * missingItems({ tracker, local, userID,  servers}: SyncOptions ): AsyncGenerator<SyncUserItemParams> {
    let localItems = new ServerUserItems(local, userID, tracker);
    let remotes: ServerUserItems[] = [...servers].map((s) => new ServerUserItems(s, userID, tracker))

    while (!localItems.hadError) {
        await Promise.all([
            localItems.fetchNext(),
            ...remotes.map(r => r.fetchNext())
        ])
        remotes = remotes.filter((r) => !r.isDone)
        if (remotes.length == 0) {
            return // No more remote items.
        }

        let localNext = localItems.headItem?.value
        let headItems = [
            localNext,
            ... remotes.map((r) => r.headItem?.value)
        ]
        let nextItems = headItems.filter((v) => !!v) as ItemListEntry[]

        nextItems.sort(entryByTimestamp);
        let first = nextItems[0]

        // Eventually, implement and check some early-end conditions here.

        if (localItems.popIfEquals(first)) {
            // Have this locally. No need to sync it.
            remotes.forEach((r) => r.popIfEquals(first))
            continue
        }

        // Find servers we could sync this from:
        let choices = remotes.filter((r) => r.popIfEquals(first))
        // This could be more intelligent. Maybe we prefer the faster server?
        // For now, just random:
        let choice = randomElement(choices)

        let signature: Signature
        try {
           signature = Signature.fromBytes(first.signature?.bytes)
        } catch (error) {
            tracker.warn(`Bad signature ${first.signature?.bytes} from server ${choice.client.url}`)
            continue
        }

        yield {
            userID,
            signature,
            from: choice.client,
            to: local, 
            tracker,
        }
    }
}

// TODO: There's an issue here where if:
// 1. Someone posts a lot of items with the exact same timestamp
// 2. and the server doesn't order it this way
// ... then we may try to sync some items that aren't necessary.
// 
// We could fix this locally by making sure to order same-timestamp items
// by their signatures. (TODO)
//
// Sort by newest first, then by signature for a full ordering.
function entryByTimestamp(e1: ItemListEntry, e2: ItemListEntry): number {
    let cmp = e2.timestamp_ms_utc - e1.timestamp_ms_utc
    if (cmp != 0) return cmp

    let e1b = e1.signature?.bytes
    let e2b = e2.signature?.bytes
    if (!e1b) return -1;
    if (!e2b) return 1;

    cmp = e2b.length - e1b.length
    if (cmp != 0) return cmp

    for (let i = 0; i < e1b.length; i++) {
        cmp = e2b[i] - e1b[i]
        if (cmp != 0) return cmp
    }

    return 0
}

function randomElement<T>(list: T[]): T {
    if (list.length == 1) {
        return list[0]
    }

    let rnd = Math.random() * list.length
    rnd = Math.floor(rnd)
    return list[rnd]
}

// Helper methods for querying/merging user items. 
class ServerUserItems {
    client: Client
    private userItems: AsyncGenerator<ItemListEntry>
    headItem: IteratorResult<ItemListEntry, void> | undefined
    
    isDone = false
    hadError = false
        
    constructor(server: string|Client, userID: UserID, public tracker: TaskTracker) {
        if (typeof server == "string") {
            this.client = new Client({base_url: server})
        } else {
            this.client = server
        }

        this.userItems = this.client.getUserItems(userID)
    }

    async fetchNext(): Promise<void> {
        if (this.headItem !== undefined) { return }
        if (this.isDone) { return }
        try {
            this.headItem = await this.userItems.next()
            if (this.headItem.done) { this.isDone = true }
        } catch (e) {
            this.tracker.warn(`error reading from ${this.client.url}. Skipping server.`)
            this.isDone = true
            this.hadError = true
        }
    }

    // If the head item is equal to this value, pop it & return true.
    popIfEquals(entry: ItemListEntry): boolean {
        if (!this.headEquals(entry)) return false
    
        this.headItem = undefined
        return true
    }

    private headEquals(entry: ItemListEntry): boolean {
        if (this.isDone) return false
        let head = this.headItem?.value
        if (!head) return false
        return entryByTimestamp(head, entry) == 0
    }

    
}

function publishMyPosts() {
    taskTracker.run("Publish my posts", publishMyPostsTask)
}

// TODO: implement a missingRemoteItems to mirror missing[Local]Items(). Use that here to optimize this.
async function publishMyPostsTask(tracker: TaskTracker) {
    let local = $appState.client
    let result = await local.getProfile(userID)
    if (!result) throw `Current user does not have a local profile.`

    let profile = result.item.profile
    let servers: Set<string>
    if (serverURL) {
        servers = new Set([serverURL])
    } else {
        servers = serversFromProfile(profile)
    }
    if (servers.size === 0) {
        throw `User profile doesn't specify any servers.`
    }

    // Loading full list once, because we may need it N times below:
    // TODO: as above, we could limit this to some shorter timespan by default.
    let localItems = await loadItemEntries(local, userID)

    await tracker.runSubtask("Syncing Items", async (tracker) => {
        for (let server of servers) {
            await tracker.runSubtask(`Syncing to ${server}`, async (tracker) => {
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
            }) // syncing to server
        } // for server
    }) // syncing items

   

    
    await tracker.runSubtask(`Syncing file attachments`, async (tracker) => {
        let bytesCopied = 0

        for (let entry of localItems.values()) {
            if (entry.type && entry.type != "post") continue // for now, only posts have attachments.
            let item = await local.getItem(entry.userID, entry.signature, {skipSignatureCheck: true})
            if (!item) {
                tracker.error(`Couldn't fetch ${entry.signature} from the local server!?`)
                continue
            }

            for (let attachment of getAttachments(item)) {
                bytesCopied += await sendAttachmentMulti({
                    userID: entry.userID,
                    signature: entry.signature,
                    fileName: attachment.name,
                    from: local,
                    toServers: servers,
                    tracker,
                })
            } // for attachments
        } // for localItems

        tracker.log(`Copied ${readableSize(bytesCopied)}`)
    })
}



</script>