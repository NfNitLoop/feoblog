<!-- Displays the homepage feed in the client. -->
<div>
    {#each items as entry, index (entry.signature)}
        <ItemView 
            userID={UserID.fromBytes(entry.user_id.bytes).toString()}
            signature={Signature.fromBytes(entry.signature.bytes).toString()}
        />
    {/each}
</div>

<script lang="ts">
import type { Writable } from "svelte/store";

import type { ItemListEntry } from "../../protos/feoblog";
import type { AppState } from "../../ts/app";
import { UserID, Signature } from "../../ts/client";

import ItemView from "../ItemView.svelte"

export let appState: Writable<AppState>
let items: ItemListEntry[] = []

async function loadData() {
    let client = $appState.client

    // TODO: Support pagination. For now, load everything.
    let list = await client.getHomepageItems()
    items = list.items
}
loadData()

</script>