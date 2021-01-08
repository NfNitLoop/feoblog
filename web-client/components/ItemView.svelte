<div>
    

{#await item}
    <p>Loading:
    <br>user_id: { user_id }
    <br>signature: { signature }
    </p>
{:then item}

    <div class="postPreview item">
        {#if item.post.title}
        <h1 class="title">{ item.post.title }</h1>
        {/if}
        <Timestamp utc_ms={item.timestamp_ms_utc} minute_offset={item.utc_offset_minutes} />
        
        {@html markdownToHtml(item.post.body)}
    </div>

{:catch error}
    <p>Error: {error}
{/await} 


</div>



<script lang="ts">
import {Client, Config} from "../ts/client"
import Timestamp from "./Timestamp.svelte"
import { markdownToHtml } from "../ts/common";

export let user_id: string = ""
export let signature: string = ""



const client = new Client({
    base_url: "", // This server
})

let item = getItem()

async function getItem() {
    let item = await client.getItem(user_id, signature)
    return item
}




</script>