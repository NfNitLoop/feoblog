<!--
    The "friend feed", which shows posts by users a given user follows.    
-->
{#if !userID}
<div class="error">
    Invalid UserID
</div>
{:else}
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
            on:enteredPage={itemEnteredScreen}
            on:leftPage={itemLeftScreen}
        />
        <!-- 
        {#if entry.item?.timestamp_ms_utc == firstVisible?.item?.timestamp_ms_utc}
            <div>👆⬆ This guy is the first visible</div>
        {/if}
        -->
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
    <VisibilityCheck on:itemVisible={() => lazyLoader?.displayMoreItems?.()} bind:visible={endIsVisible}/>
</div><!-- feed -->
{/if}

<script lang="ts">
import type { AppState } from "../../ts/app";
import type { DisplayItem } from "../../ts/client"
import type { PageEvent } from "../ItemView.svelte";
import type { Writable } from "svelte/store";

import { getContext, onDestroy } from "svelte";
import { params, query } from "svelte-hash-router"

import { FindMatchingString, SkipUsers } from "../../ts/client"
import { UserID, LazyItemLoader, ItemFilter } from "../../ts/client";

import ItemView from "../ItemView.svelte"
import VisibilityCheck from "../VisibilityCheck.svelte"
import PageHeading from "../PageHeading.svelte"
import UserIDView from "../UserIDView.svelte"
import InputBox from "../InputBox.svelte";
import { InfiniteScroll } from "../../ts/common";

let appState: Writable<AppState> = getContext("appStateStore")

let items = new InfiniteScroll<DisplayItem>({getScrollElement: () => firstVisible?.element || null})
let endIsVisible: boolean

// Assume there are more items to lazily load until we find otherwise:
let moreItems = true

let search = ""

$: userID = UserID.tryFromString($params.userID)
$: startScrollPosition = parseScrollPosition($query.ts)

$: lazyLoader = createLazyLoader(userID, filter, startScrollPosition)

function createLazyLoader(userID: UserID|null, itemFilter: ItemFilter, scrollPos: number) {
    if (!userID) { return } 
    if (lazyLoader) { lazyLoader.stop() }
    moreItems = true

    items.clear()
    const ll = new LazyItemLoader({
        client: $appState.client,
        itemEntries: $appState.client.getUserFeedItems(userID, {before: scrollPos + 1}),
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

async function updateFollowedUsers(userID: UserID|null) {

    followedUsers = []
    skippedUsers = new Set()

    if (!userID) { return }

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
// $: console.log("visible", visibleElements.map(e => e.item?.timestamp_ms_utc))

function itemEnteredScreen(event: CustomEvent<PageEvent>) {
    visibleElements = [...visibleElements, event.detail]
}

function itemLeftScreen(event: CustomEvent<PageEvent>) {
    // Could be faster but this should be a  super small array anyway.
    visibleElements = visibleElements.filter(e => e.signature.toString() != event.detail.signature)
}

$: firstVisible = getFirstVisible(visibleElements)
$: saveScrollPosition(firstVisible)

function saveScrollPosition(event: PageEvent|null) {
    if (!event) { return }
    let ts = event.item?.timestamp_ms_utc
    if (!ts) { return }

    historyThrottle.setParam("ts", `${ts}`)
}




function parseScrollPosition(ts: string): number {
    let value = new Date().valueOf() 
    try { 
        let pos = parseInt(ts) 
        if (!isNaN(pos)) {
            
            value = pos  
        } 
    }
    catch { }
    return value
}

function getFirstVisible(events: PageEvent[]): PageEvent|null {
    if (events.length == 0) { return null }


    let event = null
    for (const e of events) {
        if (!e.item) { continue }
        let ts = e.item.timestamp_ms_utc
        if (event == null || ts > event.item!.timestamp_ms_utc) {
            event = e
        }
    }

    return event
}

function toggleSkippedUser(uid: string) {
    if (skippedUsers.has(uid)) {
        skippedUsers.delete(uid)
    } else {
        skippedUsers.add(uid)
    }
    skippedUsers = skippedUsers
}


/**
 * Provide throttled access to the window.history API via timers.
 * 
 * Both Chrome and Firefox behave badly if you call the history API too frequently,
 * which can happen when a user is quickly scrolling through a page.
 * Chrome will display a warning about throttling the API, and then the URL bar
 * just doesn't update for a long while.
 * Firefox will actually throw an exception, which can be bad for Svelte's continued
 * operation.
 */
class HistoryThrottle {
    minDelayMS = 500

    #lastReplaceMs = 0

    // The last timer we started.
    #timer: number|undefined = undefined

    replaceData = {}
    replaceTitle = "FeoBlog Scroll State"
    replaceURL: string|undefined = undefined

    setParam(name: string, value: string) {
        let spaLoc = window.location.hash.substr(1)
        let parts = spaLoc.split("?")
        let params = new URLSearchParams(parts[1] ?? "")
        params.set(name, value)

        let newURL = new URL(window.location.href)
        newURL.hash = `${parts[0]}?${params}`
        this.scheduleReplaceState(newURL.toString())
    }

    scheduleReplaceState(url: string) {
        this.replaceURL = url

        let now = new Date().valueOf()
        let delta = now - this.#lastReplaceMs
        let timeToNext = this.minDelayMS - delta
        if (this.#timer) {
            // a replace is already scheduled. Just wait for that.
            return
        }
        if (timeToNext <= 0) {
            // No need to schedule, just do it now:
            this.replaceState()
            return
        }

        this.#timer = setTimeout(() => this.replaceState(), timeToNext)
    }

    private replaceState() {
        this.#lastReplaceMs = new Date().valueOf()
        this.#timer = undefined
        window.history.scrollRestoration = 'manual'
        window.history.replaceState(this.replaceData, this.replaceTitle, this.replaceURL)
    }

    cancel() {
        clearTimeout(this.#timer)
        this.#timer = undefined
    }
}
let historyThrottle = new HistoryThrottle()
onDestroy(() => {
    historyThrottle.cancel()
})

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