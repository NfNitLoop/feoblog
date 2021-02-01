<ItemView 
    userID={userID.toString()}
    signature={signature.toString()}
    showDetail={true}
    {appState}
/>

<CommentEditor 
    {appState}
    replyToUserID={userID}
    replyToSignature={signature}
    on:sendSuccess={commentSendSuccess}
/>

<!-- TODO: Fetch & show comments -->

<script lang="ts">
import type { Writable } from "svelte/store"
import type { AppState } from "../../ts/app"

import { Signature, UserID } from "../../ts/client"

import ItemView from "../ItemView.svelte"
import CommentEditor from "../CommentEditor.svelte"

// svelte-spa-router passes in URL params here:
export let params: any

export let appState: Writable<AppState>

$: userID = UserID.fromString(params.userID)
$: signature = Signature.fromString(params.signature)

function commentSendSuccess(event: any) {
    let uid = event.detail.userID
    let sig = event.detail.signature
    console.log("Send success", uid, sig)

    // TODO: Clear the editor and show the item.
}


</script>