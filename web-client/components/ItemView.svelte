<script lang="ts">
// View of a single item.
import type { Writable } from "svelte/store"
import { slide } from "svelte/transition"

import { UserID as ClientUserID} from "../ts/client"
import { markdownToHtml } from "../ts/common"
import Timestamp from "./Timestamp.svelte"
import Button from "./Button.svelte"
import type { Item, UserID } from "../protos/feoblog"
import type { AppState } from "../ts/app"
import UserIdView from "./UserIDView.svelte"

export let userID: string
export let signature: string
// Caller can provide a pre-fetched Item
export let item: Item|null|undefined = undefined
export let appState: Writable<AppState>
export let showDetail = false


let itemPromise: Promise<Item|null>

// Support routing from svelte-spa-router.
// 🙁 See: https://github.com/ItalyPaleAle/svelte-spa-router/issues/183
export let params: any|undefined
$: {
    if (params) {
        userID = params.userID
        signature = params.signature
   }
} 

let viewMode: "normal"|"markdown"|"data" = "normal"

$: {
    // Rerun getItem when any of these change:
    itemPromise = getItem(userID, signature); item
}

async function getItem(userID, signature) {
    if (item !== undefined) {
        // User has provided their own, don't load one:
        return item
    }
    return await $appState.client.getItem(userID, signature)
}

function createComment() {
    console.log("TODO: createComment() unimplemented")
}

</script>   


<div class="item">
{#await itemPromise}
    <p>Loading...
        <!-- 
    <br>user_id: { userID }
    <br>signature: { signature }
    -->
    </p>
{:then item}
        {#if !item}
            No such item: <code>/u/{userID}/i/{signature}/</code>
        {:else if item.post}
            {#if item.post.title}
            <h1 class="title">{ item.post.title }</h1>
            {/if}
            <div class="userInfo">
                <UserIdView userID={ClientUserID.fromString(userID)} {appState}/>
            </div>
            <Timestamp utc_ms={item.timestamp_ms_utc} minute_offset={item.utc_offset_minutes} href={`#/u/${userID}/i/${signature}/`} />
            
            {#if viewMode == "normal"}
                {@html markdownToHtml(item.post.body)}
            {:else if viewMode == "markdown"}
                Markdown source:
                <code><pre>{item.post.body}</pre></code>
            {:else} 
                JSON representation of Protobuf Item:
                <code><pre>{JSON.stringify(item.toObject(), null, 4)}</pre></code>
            {/if}

            {#if showDetail}
            <div>
                <Button on:click={createComment}>Comment</Button>
                {#if viewMode != "normal"}<Button on:click={() => viewMode = "normal"}>View Normal</Button>{/if}
                {#if viewMode != "markdown"}<Button on:click={() => viewMode = "markdown"}>View Markdown</Button>{/if}
                {#if viewMode != "data"}<Button on:click={() => viewMode = "data"}>View Data</Button>{/if}
            </div>
            {/if}
        {:else}
            Unknown item type.
        {/if}
{:catch error}
    <p>Error: {error}
{/await} 
</div>
