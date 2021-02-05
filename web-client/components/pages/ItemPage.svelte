<ItemView 
    userID={userID.toString()}
    signature={signature.toString()}
    showDetail={true}
    {appState}
    on:itemLoaded={itemLoaded}
/>

{#if allowComments}
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
{/if}

{#if isProfile}
<div class="item">
    <div class="body">
        <p>You are viewing a single update of this user's profile. 
            This may not be the most recent profile for this user. 
            You can always view the most recent profile at:
        </p>

        <Button href={`#/u/${userID}/profile`}>View Profile</Button>
            
    </div>
</div>
{/if}



<script lang="ts">
import type { Writable } from "svelte/store"
import type { AppState } from "../../ts/app"

import { Signature, UserID } from "../../ts/client"

import ItemView from "../ItemView.svelte"
import CommentEditor from "../CommentEditor.svelte"
import type { Item } from "../../protos/feoblog";
import Button from "../Button.svelte"

// svelte-spa-router passes in URL params here:
export let params: any
export let appState: Writable<AppState>

$: userID = UserID.fromString(params.userID)
$: signature = Signature.fromString(params.signature)
$: loadComments(allowComments, userID, signature)

let item: Item|undefined = undefined
function itemLoaded(event: CustomEvent<Item>) {
    item = event.detail
}

$: isProfile = !!(item && item.profile)
// Don't show coments UI on profile updates. Profiles are the one ephemeral-ish part of FeoBlog.
// An individual comment update never goes away, but it will not always be the newest.
$: allowComments = !!(item && !isProfile)



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

    // SignAndSend will send us to the ItemView page for the new comment.
}

// Loads all comments for now. Will add pagination & filtering later.
async function loadComments(allowComments: boolean, userID: UserID, signature: Signature) {

    replies = []

    if (!userID || !signature) {
        return
    }
    if (!allowComments) {
        return
    }

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