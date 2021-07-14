<!--
    The "friend feed", which shows posts by users a given user follows.    
-->
<div class="feed">
    <PageHeading>
        <h1>Feed for <UserIDView {userID}/></h1>

        <div slot="settings">
            <div class="searchBox">
                <input type="text" bind:value={search} placeholder="(Search)"/>
            </div>
        
            {#if followedUsers.length > 0}
            <h4>Show follows:</h4>
            <div class="follows">
                {#each followedUsers as follow}
                    <div class="follow" 
                        on:click={() => toggleSkippedUser(follow.userID.toString())}
                        class:skipped={skippedUsers.has(follow.userID.toString())}
                    >{follow.displayName}</div>
                {/each}
            </div>
            {/if}
        </div>
    </PageHeading>

    {#each items as entry (entry) }
    <ItemView 
        userID={entry.userID.toString()}
        signature={entry.signature.toString()}
        item={entry.item}
        clickable={true}
        {appState}
    />
    {/each}
    <div class="item"><div class="body">
        {#if !moreItems}
            No more items to display.
        {:else if search.trim()}
            Searching...
        {:else}
            Loading...
        {/if}
    </div></div>
    <VisibilityCheck on:itemVisible={lazyLoader.displayMoreItems} bind:visible={endIsVisible}/>
</div><!-- feed -->


<script lang="ts">
import type { Writable } from "svelte/store";

import type { AppState } from "../../ts/app";
import type { DisplayItem } from "../../ts/client"
import { FindMatchingString, SkipUsers } from "../../ts/client"
import { UserID, LazyItemLoader, ItemFilter } from "../../ts/client";

import ItemView from "../ItemView.svelte"
import VisibilityCheck from "../VisibilityCheck.svelte"
import PageHeading from "../PageHeading.svelte"
import UserIDView from "../UserIDView.svelte"

export let appState: Writable<AppState>

let items: DisplayItem[] = []
let endIsVisible: boolean

// Assume there are more items to lazily load until we find otherwise:
let moreItems = true

export let params: {
    userID: string
}

let search = ""

$: userID = UserID.fromString(params.userID)

let userDisplayName = "..."
$: updateDisplayName(userID)

async function updateDisplayName(userID: UserID) {
    userDisplayName = "..."
    let name = await $appState.getPreferredName(userID)
    userDisplayName = name || userID.toString()
}

$: lazyLoader = createLazyLoader(userID, filter)

function createLazyLoader(userID: UserID, itemFilter: ItemFilter) {
    if (lazyLoader) { lazyLoader.stop() }
    moreItems = true

    items = []
    const ll = new LazyItemLoader({
        client: $appState.client,
        itemEntries: $appState.client.getUserFeedItems(userID),
        endReached: () => { moreItems = false },

        // TODO: Fold displayItem into itemFilter.
        displayItem: (di) => {
            if (di.item.profile) {
                // Don't display profile updates here.
                return
            }
            items = [...items, di]
        },
        itemFilter,
        continueLoading: () => endIsVisible
    })

    ll.displayMoreItems()
    return ll
}

$: filter = function() { 
    let filters: ItemFilter[] = []

    // Search by string.
    // Currently searches all of markdown.
    let searchTerm = search.trim()
    if (searchTerm) {
        filters.push(new FindMatchingString(searchTerm))
    }

    // Allow filtering out specific follows if they get chatty.
    if (skippedUsers.size > 0) {
        filters.push(new SkipUsers(skippedUsers))
    }

    // TODO: Filter for items with attachments?
    // TODO: Filter out comments?

    return ItemFilter.matchAll(filters)
}()

interface FollowedUser {
    userID: UserID
    displayName: string
}

let followedUsers: FollowedUser[] = []
let skippedUsers = new Set<string>()
$: updateFollowedUsers(userID)

async function updateFollowedUsers(userID: UserID) {

    followedUsers = []
    skippedUsers = new Set()

    let client = $appState.client
    let profile
    try {
        profile = await client.getLatestProfile(userID)
    } catch (error) {
        console.error(`Error fetching profile for ${userID}`)
        return
    }

    if (!profile) return;
    let pProfile = profile.item.profile
    if (!pProfile) {
        console.error("Got a profile object that doesn't contain a profile")
        return
    }

    let newFollows: FollowedUser[] = []

    for (const follow of pProfile.follows) {
        try {
            let uid = UserID.fromBytes(follow.user.bytes)
            newFollows.push({
                userID: uid,
                displayName: await $appState.getPreferredName(uid) || uid.toString()
            })
        } catch (error) {
            console.error("Error parsing userID bytes:", follow)
        }

    }

    // A user's follow feed also includes their own posts:
    try {
        newFollows.push({
            userID,
            displayName: await $appState.getPreferredName(userID) || userID.toString()
        })
    } catch (error) {
        console.error("Error fetching preferred name for:", userID.toString())
    }

    newFollows.sort((f1, f2) => f1.displayName.localeCompare(f2.displayName))

    followedUsers = newFollows
}

function toggleSkippedUser(uid: string) {
    if (skippedUsers.has(uid)) {
        skippedUsers.delete(uid)
    } else {
        skippedUsers.add(uid)
    }
    skippedUsers = skippedUsers
}

</script>

<style>
.feed {
    max-width: 55rem;
}

.follows {
    display: grid;
    /* grid-auto-flow: row; */
    grid-template-columns: 1fr 1fr 1fr;
    gap: 0.5rem;
}

.follows .follow {
    display: block;
    border: 1px solid rgba(0, 0, 0, 0.2);
    padding: 0.25rem;

}

.follows .follow:hover, .follows .follow.skipped {
    cursor: pointer;
    background-color: rgba(0, 0, 0, 0.1)
}

.follows .follow.skipped {
    text-decoration: line-through;
}

</style>