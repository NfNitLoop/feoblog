<!-- Displays the homepage feed in the client. -->
<div>
    {#each items as entry, index (entry.signature)}
        <ItemView 
            userID={entry.userID}
            signature={entry.signature}
            item={entry.item}
            {appState}
        />
    {/each}
</div>

<script lang="ts">
import { listen } from "svelte/internal";

import type { Writable } from "svelte/store";

import type { Item, ItemListEntry } from "../../protos/feoblog";
import type { AppState } from "../../ts/app";
import { UserID, Signature } from "../../ts/client";
import { prefetch } from "../../ts/common";

import ItemView from "../ItemView.svelte"

export let appState: Writable<AppState>
let items: DisplayItem[] = []

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

async function loadData() {
    let client = $appState.client

    // TODO: Support pagination. For now, loads whatever the server gives us.
    let list = await client.getHomepageItems()

    // Prefetch for faster loading:
    function prefetchEntry(entry: ItemListEntry):  PrefetchItem {
        let userID = UserID.fromBytes(entry.user_id.bytes)
        let signature = Signature.fromBytes(entry.signature.bytes)
        return {
            item: client.getItem(userID, signature),
            userID,
            signature,
        }
    }

    for (let pfItem of prefetch(list.items, 4, prefetchEntry)) {

        let {item: itemPromise, signature, userID} = pfItem
        
        try {
            let item = await itemPromise
            if (item === null) throw "No such item"

            items.push({
                item,
                signature: signature.toString(),
                userID: userID.toString(),
            })
            items = items
        } catch (exception) {
            console.error("Error loading Item:", userID, signature, exception)
        }
    }

}
loadData()



</script>