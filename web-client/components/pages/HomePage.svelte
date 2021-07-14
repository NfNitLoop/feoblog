<!-- Displays the homepage feed in the client. -->
{#each items as entry, index (entry.signature)}
    <ItemView 
        userID={entry.userID.toString()}
        signature={entry.signature.toString()}
        item={entry.item}
        clickable={true}
        {appState}
    />
{:else}
    {#if !moreItems}
    <div class="item"><div class="body">
        {#if $appState.loggedInUser }
            Nothing to see here yet. Do you want to <a href="#/post">write a post</a>?

            <p>If you see your posts on <a href="#/u/{$appState.loggedInUser}/feed">your feed</a> but not here, 
            make sure you flag your userID with <code>--homepage</code> like this:</p>

            <code><pre>blog user add {$appState.loggedInUser} --homepage</pre></code>
        {:else}
            Nothing to see here yet. Do you want to <a href="#/login">log in</a> and write a post?
        {/if}
    </div></div>
    {/if}
{/each}

<VisibilityCheck on:itemVisible={lazyLoader.displayMoreItems} bind:visible={endIsVisible}/>

<script lang="ts">
import type { Writable } from "svelte/store";

import type { AppState } from "../../ts/app";
import type { DisplayItem } from "../../ts/client"
import { LazyItemLoader } from "../../ts/client";

import ItemView from "../ItemView.svelte"
import VisibilityCheck from "../VisibilityCheck.svelte";

export let appState: Writable<AppState>

let items: DisplayItem[] = []
let endIsVisible: boolean


// Assume there are more items to lazily load until we find otherwise:
let moreItems = true

// 

$: lazyLoader = createLazyLoader()
function createLazyLoader() {
    items = []
    if (lazyLoader) { lazyLoader.stop() }

    return new LazyItemLoader({
        itemEntries: $appState.client.getHomepageItems(),
        client: $appState.client,
        continueLoading: () => endIsVisible,
        endReached: () => { moreItems = false },
        displayItem: (di) => {
            // Neither comments nor profile updates belong on the homepage.
            if (di.item.post) {
                items = [...items, di]
            }
        }
    })
}


</script>