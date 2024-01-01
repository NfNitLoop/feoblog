<!--
    Used both in ItemView and CommentEditor.
    Does NOT include a <div class="item"> around it.
-->
<div use:fixLinks={{mode: linkMode}}>
    <ItemHeader {showReplyTo} {item} {signature} {userID} bind:viewMode />
    <div class="body">
        {#if viewMode == "normal"}
            {@html markdownToHtml(comment.text, {stripImages: true, relativeBase: `/u/${userID}/i/${signature}/`})}
        {:else if viewMode == "markdown"}
            <p>Markdown source:</p>
            <code><pre>{comment.text}</pre></code>
        {:else} 
            <p>JSON representation of Protobuf Item:</p>
            <code><pre>{JSON.stringify(item, null, 4)}</pre></code>
        {/if}

    </div>  
</div>

<script lang="ts">
import type { Writable } from "svelte/store";

import type { AppState } from "../ts/app";

import type {UserID, protobuf as pb} from "../ts/client"
import { getInner } from "../ts/client"
import {markdownToHtml, fixLinks} from "../ts/common"

import ItemHeader from "./ItemHeader.svelte"
import type {ViewMode} from "./ItemHeader.svelte"
import { getContext } from "svelte";

let appState: Writable<AppState> = getContext("appStateStore")
export let item: pb.Item
export let showReplyTo = true

export let userID:UserID
export let linkMode: "fix"|"ignore"|"newWindow" = "fix"

let viewMode: ViewMode = "normal"

// If we want to use this as a preview, we must account for an invalid signature:
export let signature: string

$: comment = getInner(item, "comment")!

</script>

