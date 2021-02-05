<!--
    The "friend feed", which shows posts by users a given user follows.    
-->
<!-- Displays the homepage feed in the client. -->
<div class="feed">
{#each items as entry (entry) }
    <ItemView 
        userID={entry.userID.toString()}
        signature={entry.signature.toString()}
        item={entry.item}
        clickable={true}
        {appState}
    />
{:else}
<div class="item">
    {#if !moreItems}
        Nothing to see here. You may need to <a href="#/my_profile">edit your profile</a>
        to follow people, or <a href="#/post">write your first post</a>.
    {:else}
        Loading...
    {/if}
</div>
{/each}
<VisibilityCheck on:itemVisible={lazyLoader.displayMoreItems} bind:visible={endIsVisible}/>
</div>


<script lang="ts">
import type { Writable } from "svelte/store";

import type { AppState } from "../../ts/app";
import type { DisplayItem } from "../../ts/client"
import { UserID, LazyItemLoader } from "../../ts/client";

import ItemView from "../ItemView.svelte"
import VisibilityCheck from "../VisibilityCheck.svelte";

export let appState: Writable<AppState>

let items: DisplayItem[] = []
let endIsVisible: boolean

// Assume there are more items to lazily load until we find otherwise:
let moreItems = true

export let params: {
    userID: string
}

$: userID = UserID.fromString(params.userID)


$: lazyLoader = createLazyLoader(userID)

function createLazyLoader(userID: UserID) {
    items = []
    return new LazyItemLoader({
        client: $appState.client,
        endIsVisible: () => endIsVisible,
        itemEntries: $appState.client.getUserFeedItems(userID),
        endReached: () => { moreItems = false },
        displayItem: (di) => {
            if (di.item.profile) {
                // Don't display profile updates here.
                return
            }
            items = [...items, di]
        },
    })
}

</script>

<style>
    .feed {
        max-width: 55rem;
    }
</style>