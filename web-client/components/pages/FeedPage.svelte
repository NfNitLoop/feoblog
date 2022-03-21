<!--
    The "friend feed", which shows posts by users a given user follows.    
-->
{#if !userID}
<div class="error">
    Invalid UserID
</div>
{:else}
<div class="feed">
    <PageHeading breadcrumbs={{crumbs: [{userID}, {text: "Feed"}]}} navItems={getNav(userID)}>
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

    <ItemsScroll
        {createItemLoader}
        {scrollPos}
        itemFilter={filter}
    />

</div><!-- feed -->
{/if}

<script lang="ts">
import type { AppState } from "../../ts/app";
import type { ItemOffsetParams } from "../../ts/client"
import type { Writable } from "svelte/store";

import { getContext } from "svelte";
import { params, query } from "svelte-hash-router"

import { FindMatchingString, SkipUsers, ExcludeItemTypes} from "../../ts/client"
import { UserID, ItemFilter } from "../../ts/client";

import PageHeading from "../PageHeading.svelte"
import UserIDView from "../UserIDView.svelte"
import InputBox from "../InputBox.svelte";
import ItemsScroll from "../ItemsScroll.svelte";
import { ItemType } from "../../protos/feoblog";

let appState: Writable<AppState> = getContext("appStateStore")


let search = ""

$: userID = UserID.tryFromString($params.userID)
$: scrollPos = parseScrollPosition($query.ts)

function createItemLoader(params: ItemOffsetParams) {
    if (!userID) return null
    return $appState.client.getUserFeedItems(userID, params)
}

function getNav(userID: UserID) {
    let app = $appState
    let nav = app.navigator
    let loggedIn = userID.toString() == app.loggedInUser?.toString()
    if (loggedIn) {
        return [
            { text: "New Post", href: nav.newPost().hash },
            { text: "Sync", href: nav.sync().hash }
        ]
    }
}


$: filter = function() { 
    let filters: ItemFilter[] = [
        // TODO: Filter out comments and/or posts?
        // TODO: Show profile updates. (Once we've got a profile delta viewer)
        new ExcludeItemTypes([ItemType.PROFILE])
    ]

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