// Logic for syncing FeoBlog Items to/from different servers.

import type { TaskTracker } from "./common"

// TODO: "Client" is a Node type and not importing this can result in unexpected class resolution. >.<
import { Client, ProfileResult, UserID } from "./client"
import type { Item, ItemListEntry } from "../protos/feoblog"

export interface SyncUserArgs {
    tracker: TaskTracker
    local: Client
    userID: UserID

    // Remote servers to sync from
    servers: Set<string>

    opts: SyncOptions
}

export interface SyncOptions {
    localClient: Client

    // If set, boostrap sync from only this server:
    sourceServerUrl?: string

    // Stop syncing after we already have this many recent items (of any type)
    // Can set to Number.POSITIVE_INFINITY to sync everything.
    recentItems: number

    // If present, do a backfill:
    backfill?: BackfillOptions
}

export interface BackfillOptions {
    // TODO
    // Backfill only back to this date.
    toDateUtcMs?: number

    // TODO
    // If present, only allocate this many bytes for backfilling attachments.
    // Set to 0 to disable backfilling attachments.
    maxAttachmentBytesTotal?: number
}


export interface SyncUserProfileArgs extends SyncUserArgs {
    // the local profile has likely already been loaded before trying to sync it (so we know what servers to sync from)
    // You can provide it here to save another request to the server.
    localProfile?: ProfileResult | null
}


/**
 * Sync a user's profile from remote servers to the local one.
 * Returns true if a profile was sync'd.
 */
export async function syncUserProfile(args: SyncUserProfileArgs): Promise<boolean> {

    let {tracker, local, userID, localProfile, servers} = args

    if (localProfile === undefined) {
        // Profile wasn't passed in, try loading it:
        localProfile = await local.getProfile(userID)
    }

    let loadProfile = async (remote: Client) => {
        try { 
            return await remote.getProfile(userID)
        } catch (error: unknown) {
            tracker.warn(`Error fetching profile from remote ${remote.url}`)
            return null
        }
    }

    let remoteClients = [...servers].map(
        s => new Client({base_url: s})
    )

    let remoteProfiles = (
            await Promise.all(
                remoteClients.map(loadProfile)
            )
        ).filter(notNull)

    if (remoteProfiles.length == 0) {
        tracker.warn("No profile found.")
        return false
    }

    let allProfiles = [localProfile, ...remoteProfiles].filter(notNull)
    allProfiles.sort(resultByTimestampSigDesc)
    let newest = allProfiles[0]
    
    if (newest.signature.asBase58 !== localProfile?.signature.asBase58) {
        await local.putItem(userID, newest.signature, newest.bytes)
        return true
    }

    return false
}


// Sort by (timestamp, signature) descending.
export function entryByTimestampSigDesc(e1: ItemListEntry, e2: ItemListEntry): number {
    let cmp = e2.timestamp_ms_utc - e1.timestamp_ms_utc
    if (cmp != 0) return cmp

    let e1b = e1.signature?.bytes
    let e2b = e2.signature?.bytes
    if (!e1b) {
        console.error("ItemListEntry missing signature bytes:", e1);
        return 0;
    }
    if (!e2b) {
        console.error("ItemListEntry missing signature bytes:", e2);
        return 0;
    }

    if (e2b.length != e1b.length) {
        console.error("Can not compare signatures of different length", e1b, e2b)
        return 0;
    }

    for (let i = 0; i < e1b.length; i++) {
        cmp = e2b[i] - e1b[i]
        if (cmp != 0) return cmp
    }

    return 0
}

function resultByTimestampSigDesc(lhs: ProfileResult, rhs: ProfileResult) {
    let cmp = rhs.item.timestamp_ms_utc - lhs.item.timestamp_ms_utc
    if (cmp != 0) return cmp

    let lhBytes = lhs.signature.bytes
    let rhBytes = rhs.signature.bytes

    for (let i = 0; i < lhBytes.length; i++) {
        cmp = rhBytes[i] - lhBytes[i]
        if (cmp != 0) return cmp
    }

    return 0
}

// Feels like TypeScript should know this as a "type guard" w/o me needing to specify a type predicate?
// See: https://www.typescriptlang.org/docs/handbook/2/narrowing.html#using-type-predicates
function notNull<T>(value: T|null): value is T {
    return value !== null
}


