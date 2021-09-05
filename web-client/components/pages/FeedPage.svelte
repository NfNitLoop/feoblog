<!--
    The "friend feed", which shows posts by users a given user follows.    
-->
<div class="feed">
    <PageHeading>
        <h1>Feed for <UserIDView {userID}/></h1>

        <div slot="settings">
            <div class="searchBox">
                <InputBox bind:value={search} placeholder="Search"/>
            </div>
        
            {#if followedUsers.length > 0}
            <h4>Filter follows:</h4>
            <div class="follows">
                {#each followedUsers as follow}
                    <div class="follow" 
                        on:click={() => toggleSkippedUser(follow.userID.toString())}
                        class:skipped={skippedUsers.has(follow.userID.toString())}
                    ><input type="checkbox" checked={!skippedUsers.has(follow.userID.toString())}> {follow.displayName}</div>
                {/each}
            </div>
            {/if}
        </div>
    </PageHeading>

    {#each $items as entry (entry) }
    <ItemView 
        userID={entry.userID.toString()}
        signature={entry.signature.toString()}
        item={entry.item}
        {appState}
        on:enteredPage={itemEnteredScreen}
        on:leftPage={itemLeftScreen}
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
import type { PageEvent } from "../ItemView.svelte";
import { FindMatchingString, SkipUsers } from "../../ts/client"
import { UserID, LazyItemLoader, ItemFilter } from "../../ts/client";

import ItemView from "../ItemView.svelte"
import VisibilityCheck from "../VisibilityCheck.svelte"
import PageHeading from "../PageHeading.svelte"
import UserIDView from "../UserIDView.svelte"
import InputBox from "../InputBox.svelte";
import { InfiniteScroll } from "../../ts/common";

export let appState: Writable<AppState>


let items = new InfiniteScroll<DisplayItem>({getScrollElement})
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

    items.clear()
    const ll = new LazyItemLoader({
        client: $appState.client,
        itemEntries: $appState.client.getUserFeedItems(userID),
        endReached: () => { moreItems = false },

        displayItem: async (di) => {
            // TODO: Fold displayItem into itemFilter.
            if (di.item.profile) {
                // Don't display profile updates here.
                return
            }
            await items.pushBottom(di)
        },
        itemFilter,
        continueLoading: () => { 
            return endIsVisible
        }
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

let visibleElements: PageEvent[] = []

function itemEnteredScreen(event: CustomEvent<PageEvent>) {
    visibleElements = [...visibleElements, event.detail]
}

function itemLeftScreen(event: CustomEvent<PageEvent>) {
    // Could be faster but this should be a  super small array anyway.
    visibleElements = visibleElements.filter(e => e.signature.toString() != event.detail.signature)
}

function getScrollElement(): HTMLElement|null {
    if (visibleElements.length == 0) { return null }

    let element = null
    let timestamp = null
    for (const el of visibleElements) {
        if (!el.item) { continue }
        let ts = el.item.timestamp_ms_utc
        if (timestamp == null || ts < timestamp) {
            element = el.element
            timestamp = ts
        }
    }

    return element
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

.follows {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
}




.follow {
    white-space: nowrap;
    /* TODO: Overflow */
}


.follow {
    display: block;
    padding: 0.25rem 0.75rem;
    padding-left: 0.25rem;
    border-radius: 3px;
    background-color: #eee;
    user-select: none;
    overflow: hidden;
    text-overflow: ellipsis;
}

.follow input[type="checkbox"] {
    margin: 4px;
}

.follows .follow:hover, .follows .follow.skipped {
    cursor: pointer;
    background-color: rgba(0, 0, 0, 0.1)
}

.follows .follow.skipped {
    color: #888;
}

.searchBox :global(input[type="text"]) {
    background-color: #eee;
}

h4 {
    margin-bottom: 0.2em;
}

</style>