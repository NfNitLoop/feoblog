<script lang="ts">
// View of a single item.

import {Client, Config} from "../ts/client"
import { markdownToHtml } from "../ts/common";
import Timestamp from "./Timestamp.svelte"

export let userID: string|undefined
export let signature: string|undefined

// Support routing from svelte-spa-router.
// 🙁 See: https://github.com/ItalyPaleAle/svelte-spa-router/issues/183
export let params: any|undefined
$: userID = params?.userID
$: signature = params?.signature

// TODO: Accept a client instead. and/or use appstate?
const client = new Client({
    base_url: "", // This server
})

$: item = getItem(userID, signature)

async function getItem(userID, signature) {
    return await client.getItem(userID, signature)
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
        {:else}
            {#if item.post.title}
            <h1 class="title">{ item.post.title }</h1>
            {/if}
            <Timestamp utc_ms={item.timestamp_ms_utc} minute_offset={item.utc_offset_minutes} />
            
            {@html markdownToHtml(item.post.body)}
        {/if}
{:catch error}
    <p>Error: {error}
{/await} 
</div>



