<script lang="ts">
// View of a single item.

import {Client, Config} from "../ts/client"
import { markdownToHtml } from "../ts/common";
import Timestamp from "./Timestamp.svelte"
import Button from "./Button.svelte"

export let userID: string|undefined
export let signature: string|undefined

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

// TODO: Accept a client instead. and/or use appstate?
const client = new Client({
    base_url: "", // This server
})

$: item = getItem(userID, signature)

async function getItem(userID, signature) {
    return await client.getItem(userID, signature)
}

function createComment() {
    console.log("TODO: createComment() unimplemented")
}

</script>   


<div class="item">
{#await item}
    <p>Loading:
    <br>user_id: { userID }
    <br>signature: { signature }
    </p>
{:then item}
        {#if !item}
            No such item: <code>/u/{userID}/i/{signature}/</code>
        {:else if item.post}
            {#if item.post.title}
            <h1 class="title">{ item.post.title }</h1>
            {/if}
            <Timestamp utc_ms={item.timestamp_ms_utc} minute_offset={item.utc_offset_minutes} />
            
            {#if viewMode == "normal"}
                {@html markdownToHtml(item.post.body)}
            {:else if viewMode == "markdown"}
                Markdown source:
                <code><pre>{item.post.body}</pre></code>
            {:else} 
                JSON representation of Protobuf Item:
                <code><pre>{JSON.stringify(item.toObject(), null, 4)}</pre></code>
            {/if}

            <div>
                <Button on:click={createComment}>Comment</Button>
                {#if viewMode != "normal"}<Button on:click={() => viewMode = "normal"}>View Normal</Button>{/if}
                {#if viewMode != "markdown"}<Button on:click={() => viewMode = "markdown"}>View Markdown</Button>{/if}
                {#if viewMode != "data"}<Button on:click={() => viewMode = "data"}>View Data</Button>{/if}
            </div>
        {:else}
            Unknown item type.
        {/if}
{:catch error}
    <p>Error: {error}
{/await} 
</div>
