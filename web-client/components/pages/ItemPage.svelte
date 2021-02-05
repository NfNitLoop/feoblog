<ItemView 
    userID={userID.toString()}
    signature={signature.toString()}
    showDetail={true}
    {appState}
/>

<div class="commentSection">
    <CommentEditor 
        {appState}
        replyToUserID={userID}
        replyToSignature={signature}
        on:sendSuccess={commentSendSuccess}
    />

    {#each replies as reply (reply)}
        <ItemView
            userID={reply.userID.toString()}
            signature={reply.signature.toString()}
            item={reply.item}
            clickable={true}
            showReplyTo={false}
            {appState}
        />
    {/each}
</div>


<script lang="ts">
import type { Writable } from "svelte/store"
import type { AppState } from "../../ts/app"

import { Signature, UserID } from "../../ts/client"

import ItemView from "../ItemView.svelte"
import CommentEditor from "../CommentEditor.svelte"
import type { Item } from "../../protos/feoblog";

// svelte-spa-router passes in URL params here:
export let params: any

export let appState: Writable<AppState>

$: userID = UserID.fromString(params.userID)
$: signature = Signature.fromString(params.signature)
$: loadComments(userID, signature)

type DisplayItem = {
    item: Item
    userID: UserID
    signature: Signature
}

let replies: DisplayItem[] = []

function commentSendSuccess(event: any) {
    let uid = event.detail.userID
    let sig = event.detail.signature
    console.log("Send success", uid, sig)

    // TODO: Clear the editor and show the item.
}

// Loads all comments for now. Will add pagination & filtering later.
async function loadComments(userID: UserID, signature: Signature) {
    if (!userID || !signature) {
        console.log("uid/sig not loaded yet")
        return
    }

    replies = []
    let client = $appState.client
    let items = client.getReplyItems(userID, signature)

    for await (let entry of items) {
        let uid = UserID.fromBytes(entry.user_id.bytes)
        let sig = Signature.fromBytes(entry.signature.bytes)

        // TODO: Could filter items here according to preferences.
        // ex: 
        // * only people I/OP follow.
        // * Only comments or (in the future) "reply posts". 

        let item
        try {
            item = await client.getItem(uid, sig)
        } catch (e) {
            console.error("Error fetching item", e)
            continue
        }
        if (!item) {
            console.error(`Item listed, but not found on server: ${uid}, ${sig}`)
            continue
        }

        let reply = {
            item,
            userID: uid,
            signature: sig,
        }
        replies = [...replies, reply]
    }
}

</script>

<style>

</style>