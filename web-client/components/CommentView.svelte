<!--
    Used both in ItemView and CommentEditor.
    Does NOT include a <div class="item"> around it.
-->
<div class="comment" use:fixLinks={{mode: linkMode}}>
<div class="userInfo">
    <UserIdView {userID} {appState}/>
    {#if showReplyTo}
        replying to <UserIdView 
            userID={UserID.fromBytes(item.comment.reply_to.user_id.bytes)}
            href={refToLink(item.comment.reply_to)}
            {appState}
        />
    {/if}
</div>
<Timestamp utc_ms={item.timestamp_ms_utc} minute_offset={item.utc_offset_minutes} href={timestampLink} />
{@html markdownToHtml(item.comment.text, {stripImages: true})}
</div>

<script lang="ts">
import type { Writable } from "svelte/store";

import type { Item, ReplyRef } from "../protos/feoblog";
import type { AppState } from "../ts/app";

import {UserID, Signature} from "../ts/client"
import {markdownToHtml, fixLinks} from "../ts/common"

import UserIdView from "./UserIDView.svelte"
import Timestamp from "./Timestamp.svelte"

export let appState: Writable<AppState>
export let item: Item
export let showReplyTo = true

export let userID:UserID
export let linkMode: "fix"|"ignore"|"newWindow" = "fix"

// If we want to use this as a preview, we must account for an invalid signature:
export let signature: string

$: parsedSignature = function() {
    try {
        return Signature.fromString(signature)
    } catch (_) {}
    return null
}()

$: timestampLink = parsedSignature ? `#/u/${userID}/i/${signature}/` : undefined

function refToLink(ref: ReplyRef): string {
    let uid = UserID.fromBytes(ref.user_id.bytes)
    let sig = Signature.fromBytes(ref.signature.bytes)
    return `#/u/${uid}/i/${sig}/`
}

</script>