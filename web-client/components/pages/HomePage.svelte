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
        {#if $appState.loggedInUser }
        <div class="item">
            Nothing to see here yet. Do you want to <a href="#/post">write a post</a>?

            <p>If you see your posts on <a href="#/u/{$appState.loggedInUser}/feed">your feed</a> but not here, 
            make sure you flag your userID with <code>--homepage</code> like this:

            <code><pre>blog user add {$appState.loggedInUser} --homepage</pre></code>
        </div>
        {:else}
        <div class="item">
            Nothing to see here yet. Do you want to <a href="#/login">log in</a> and write a post?
        </div>
        {/if}
    {/if}
{/each}

<VisibilityCheck on:itemVisible={displayMoreItems} bind:visible={endIsVisible}/>

<script lang="ts">
import { tick } from "svelte/internal";

import type { Writable } from "svelte/store";

import type { Item, ItemList, ItemListEntry } from "../../protos/feoblog";
import type { AppState } from "../../ts/app";
import { UserID, Signature } from "../../ts/client";
import { ConsoleLogger, prefetch } from "../../ts/common";

import ItemView from "../ItemView.svelte"
import VisibilityCheck from "../VisibilityCheck.svelte";

export let appState: Writable<AppState>

let items: DisplayItem[] = []
let lazyItems: AsyncIterator<DisplayItem> = getDisplayItems()
let endIsVisible: boolean

let log = new ConsoleLogger()

// Assume there are more items to lazily load until we find otherwise:
let moreItems = true

class DisplayItem {
    item: Item
    userID: string
    signature: string
}

// Whenever we change lazyItems:
$: displayInitialItems(lazyItems)
async function displayInitialItems(lazyItems: AsyncIterator<DisplayItem>) {
    items = []
}

async function displayMoreItems() {
    log.debug("displayMoreItems, endIsVisible", endIsVisible)
    while(endIsVisible) {

        let n = await lazyItems.next()
        if (n.done) {
            moreItems = false
            return
        }

        log.debug("showing 1 more item")
        items = [...items, n.value]

        // Wait for Svelte to apply state changes.
        // MAY cause endIsVisibile to toggle off, but at least in Firefox that
        // doesn't always seem to have happened ASAP.
        // I don't mind loading a few more items than necessary, though.
        await tick()
    }
}

async function* getDisplayItems(): AsyncGenerator<DisplayItem> {

    // Prefetch for faster loading:
    let entries = prefetch($appState.client.getHomepageItems(), 4, fetchDisplayItem)

    for await (let item of entries) {
        // We've already logged nulls.
        // TODO: Maybe display some placeholder instead?
        if (item !== null) yield item
    }
}

async function fetchDisplayItem(entry: ItemListEntry): Promise<DisplayItem|null> {
    let userID = UserID.fromBytes(entry.user_id.bytes)
    let signature = Signature.fromBytes(entry.signature.bytes)
    let item: Item|null 
    try {
        item = await $appState.client.getItem(userID, signature)
    } catch (e) {
        log.error("Error loading Item:", userID, signature, e)
        return null
    }

    if (item === null) {
        // TODO: Display some placeholder?
        // It does seem like an error, the server told us about the item, but doesn't have it?
        log.error("No such item", userID, signature)
        return null
    }

    return {
        item,
        signature: signature.toString(),
        userID: userID.toString(),
    }
}

</script>