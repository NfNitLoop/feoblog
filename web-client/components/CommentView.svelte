<!--
    Used both in ItemView and CommentEditor.
    Does NOT include a <div class="item"> around it.
-->
<div use:fixLinks={{mode: linkMode}}>
    <ItemHeader {appState} {showReplyTo} {item} {signature} {userID} />
    <div class="body">
        {@html markdownToHtml(item.comment.text, {stripImages: true})}
    </div>  
</div>

<script lang="ts">
import type { Writable } from "svelte/store";

import type { Item } from "../protos/feoblog";
import type { AppState } from "../ts/app";

import type {UserID} from "../ts/client"
import {markdownToHtml, fixLinks} from "../ts/common"

import ItemHeader from "./ItemHeader.svelte";

export let appState: Writable<AppState>
export let item: Item
export let showReplyTo = true

export let userID:UserID
export let linkMode: "fix"|"ignore"|"newWindow" = "fix"

// If we want to use this as a preview, we must account for an invalid signature:
export let signature: string

</script>

