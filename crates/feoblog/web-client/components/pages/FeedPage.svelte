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
        itemFilter={filter}
    />

</div><!-- feed -->
{/if}

<script lang="ts">
import type { AppState } from "../../ts/app";
import { getInner, ItemOffsetParams } from "../../ts/client"
import type { Writable } from "svelte/store";

import { getContext } from "svelte";
import { params, query } from "svelte-hash-router"

import { FindMatchingString, SkipUsers, ExcludeItemTypes, protobuf as pb} from "../../ts/client"
import { UserID, ItemFilter } from "../../ts/client";

import PageHeading from "../PageHeading.svelte"
import InputBox from "../InputBox.svelte";
import ItemsScroll from "../ItemsScroll.svelte";
import { ConsoleLogger } from "../../ts/common";

let appState: Writable<AppState> = getContext("appStateStore")
let search = ""

const logger = new ConsoleLogger({prefix: "<FeedPage>"}) // .withDebug()

    
$: userID = UserID.tryFromString($params.userID)


function createItemLoader(params: ItemOffsetParams): AsyncGenerator<pb.ItemListEntry> {
    if (!userID) throw new Error("this page requires a userID")
    let lazyItems = $appState.client.getUserFeedItems(userID, params)

    // async function * loggingLazyItems(): AsyncGenerator<ItemListEntry> {
    //     for await (let entry of lazyItems) {
    //         logger.debug("got entry at ts", entry.timestamp_ms_utc)
    //         yield entry
    //     }
    // }

    return lazyItems
}

$: filter = function() { 
    let filters: ItemFilter[] = [
        // TODO: Filter out comments and/or posts?
        // TODO: Show profile updates. (Once we've got a profile delta viewer?)
        new ExcludeItemTypes([pb.ItemType.PROFILE])
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
        profile = await client.getProfile(userID)
    } catch (error) {
        logger.error(`Error fetching profile for ${userID}`)
        return
    }

    if (!profile) return;
    let pProfile = getInner(profile.item, "profile")
    if (!pProfile) {
        logger.error("Got a profile object that doesn't contain a profile")
        return
    }

    let newFollows: FollowedUser[] = []

    for (const follow of pProfile.follows) {
        try {
            let uid = UserID.fromBytes(follow.user!.bytes)
            newFollows.push({
                userID: uid,
                displayName: await $appState.getPreferredName(uid) || uid.toString()
            })
        } catch (error) {
            logger.error("Error parsing userID bytes:", follow)
        }

    }

    // A user's follow feed also includes their own posts:
    try {
        newFollows.push({
            userID,
            displayName: await $appState.getPreferredName(userID) || userID.toString()
        })
    } catch (error) {
        logger.error("Error fetching preferred name for:", userID.toString())
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