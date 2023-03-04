<!-- Displays the homepage feed in the client. -->
<PageHeading />

<ItemsScroll {createItemLoader} {itemFilter}>
<div slot="whenEmpty">
    <div class="item"><div class="body">
        {#if $appState.loggedInUser }
            <p>Nothing to see here yet. Do you want to <a href="#/u/{$appState.loggedInUser}/post">write a post</a>?</p>

            <p>If you have <a href="#/u/{$appState.loggedInUser}/">written posts</a> but do not see them here, 
            make sure you flag your userID with <code>--homepage</code> like this:</p>

            <code><pre>feoblog user add {$appState.loggedInUser} --homepage</pre></code>
        {:else}
            <p>Nothing to see here yet. Do you want to <a href="#/login">log in</a> and write a post?</p>
        {/if}
    </div></div>
</div>
</ItemsScroll>

<script lang="ts">
import { getContext } from "svelte";
import type { Writable } from "svelte/store";

import type { AppState } from "../../ts/app";
import { ItemFilter, ItemOffsetParams, protobuf as pb } from "../../ts/client"
import ItemsScroll from "../ItemsScroll.svelte";
import PageHeading from "../PageHeading.svelte";

let appState: Writable<AppState> = getContext("appStateStore")

async function * createItemLoader(opts: ItemOffsetParams): AsyncGenerator<pb.ItemListEntry> {
    yield* $appState.client.getHomepageItems(opts)
}

// Just show whatever the server thinks should be on the homepage.
// (It's usually just posts, not comments/profile updates/etc.)
let itemFilter = ItemFilter.allowAll()

</script>