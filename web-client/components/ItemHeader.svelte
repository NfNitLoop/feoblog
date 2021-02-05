<!--
    The .header inside of an .item
-->
<div class="header">
    <ProfileImage {userID} />
    <div class="text">
        <UserIdView {userID} {appState}/>
        {#if showReplyTo && item.comment != null}
            <span>replied to</span>
            <UserIdView 
                userID={UserID.fromBytes(item.comment.reply_to.user_id.bytes)}
                href={refToLink(item.comment.reply_to)}
                {appState}
            />
        {:else if item.profile} 
            <span>updated their profile</span>
        {/if}
        <Timestamp utc_ms={item.timestamp_ms_utc} minute_offset={item.utc_offset_minutes} href={timestampLink} showRelative={!previewMode} />
    </div>
</div>

<svelte:options immutable/>
<script lang="ts">
import type { Writable } from "svelte/store";
import type { Item, ReplyRef } from "../protos/feoblog";
import type { AppState } from "../ts/app";
import { Signature, UserID } from "../ts/client";

import UserIdView from "./UserIDView.svelte"
import Timestamp from "./Timestamp.svelte"
import ProfileImage from "./ProfileImage.svelte";

// required:
export let appState: Writable<AppState>
export let userID: UserID
export let signature: string // could be invalid during preview
export let item: Item
export let previewMode = false

// optional:
export let showReplyTo = true

function refToLink(ref: ReplyRef): string {
    let uid = UserID.fromBytes(ref.user_id.bytes)
    let sig = Signature.fromBytes(ref.signature.bytes)
    return `#/u/${uid}/i/${sig}/`
}

$: timestampLink = parsedSignature ? `#/u/${userID}/i/${signature}/` : undefined
$: parsedSignature = function() {
    try {
        return Signature.fromString(signature)
    } catch (_) {}
    return null
}()

</script>

