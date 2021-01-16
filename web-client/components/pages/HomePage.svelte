<!-- Displays the homepage feed in the client. -->
{#each items as entry, index (entry.signature)}
    <ItemView 
        userID={entry.userID}
        signature={entry.signature}
        item={entry.item}
        {appState}
    />
{/each}

<VisibilityCheck on:itemVisible={displayMoreItems} bind:visible={endIsVisible}/>

<script lang="ts">
import { listen, tick } from "svelte/internal";

import type { Writable } from "svelte/store";

import type { Item, ItemList, ItemListEntry } from "../../protos/feoblog";
import type { AppState } from "../../ts/app";
import { UserID, Signature } from "../../ts/client";
import { ConsoleLogger, prefetch, prefetchAsync } from "../../ts/common";

import ItemView from "../ItemView.svelte"
import VisibilityCheck from "../VisibilityCheck.svelte";

export let appState: Writable<AppState>

let items: DisplayItem[] = []
let lazyItems: AsyncIterator<DisplayItem> = getDisplayItems()
let endIsVisible: boolean

let log = new ConsoleLogger()

class DisplayItem {
    item: Item
    userID: string
    signature: string
}

class PrefetchItem {
    item: Promise<Item|null>
    userID: UserID
    signature: Signature
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
            log.debug("DisplayMoreItems: no more to display")
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
    function prefetchEntry(entry: ItemListEntry):  PrefetchItem {
        let userID = UserID.fromBytes(entry.user_id.bytes)
        let signature = Signature.fromBytes(entry.signature.bytes)
        return {
            item: $appState.client.getItem(userID, signature),
            userID,
            signature,
        }
    }

    let entries = prefetchAsync(getEntries(), 4, prefetchEntry)
    for await (let pfItem of entries) {
        let {item: itemPromise, signature, userID} = pfItem
        
        try {
            let item = await itemPromise
            if (item === null) throw "No such item"

            yield {
                item,
                signature: signature.toString(),
                userID: userID.toString(),
            }
        } catch (exception) {
            log.error("Error loading Item:", userID, signature, exception)
        }
    }
}

async function* getEntries(): AsyncGenerator<ItemListEntry> {
    for await (let itemList of getHomepageItems()) {
        log.log("Got ItemList")
        for (let entry of itemList.items) {
            yield entry
        }
    }
}

async function* getHomepageItems(): AsyncGenerator<ItemList> {
    let before: number|undefined = undefined
    try {
        let list = await $appState.client.getHomepageItems(before)

        if (list.items.length == 0) {
            // There are no more items.
            return
        }

        yield list
        
        if (list.no_more_items) {
            return
        }

        before = list.items[list.items.length - 1].timestamp_ms_utc
    } catch (e) {
        log.error("Error calling client.getHomepageItems", e)
        return
    }
}


</script>