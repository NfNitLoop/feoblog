{#if hasText && isLoggedIn}
<TabBar tabs={["Edit", "Preview"]} bind:activeTab={currentView}/>
{/if}

{#if currentView == "Edit"}
    <div class="item">
        <div class="body">
            <ExpandingTextarea size="oneLine" {placeholder} disabled={!isLoggedIn} bind:value={text}/>
            {#if hasText}
                <SignAndSend
                    item={commentItem}
                    {appState}
                    errors={errors}
                />
            {/if}
        </div>
    </div>
{:else}
    {#if userID != null}
    <!-- TODO: Just replace with ItemView: -->
    <div class="item">
        <CommentView {appState} 
            item={commentItem}
            {userID}
            signature="unknown"
            linkMode="newWindow"
        />
    </div>
    {:else}
    <div class="item">
        <div class="body error">
            Shouldn't be able to get here. You must log in to comment & preview.
        </div>
    </div>
    {/if}
{/if}

<script lang="ts">
import { DateTime } from "luxon";
import type { Writable } from "svelte/store";
import { Item, Comment, ReplyRef, UserID as ProtoUserID, Signature as ProtoSignature} from "../protos/feoblog";
import type { AppState } from "../ts/app";
import type { Signature, UserID } from "../ts/client";
import CommentView from "./CommentView.svelte";
import ExpandingTextarea from "./ExpandingTextarea.svelte";
import SignAndSend from "./SignAndSend.svelte";
import TabBar from "./TabBar.svelte";


export let appState: Writable<AppState>
export let replyToUserID: UserID
export let replyToSignature: Signature

let currentView: "Edit"|"Preview" = "Edit"

let text = ""
$: errors = !hasText ? ["Can not submit an empty comment"] : []

$: 
{
    // Whenever any of these change, clear the text:
    // This avoids an issue where the box gets re-used on the next item page.
    $appState; replyToUserID; replyToSignature
    clear()
}

export function clear() {
    text = ""
    currentView = "Edit"
}

$: userID = $appState.loggedInUser
$: isLoggedIn = userID != null
$: hasText = text.trim().length > 0
$: placeholder = isLoggedIn ? "Leave a Comment" : "Must log in to comment"

$: commentItem = function() {
    let item = new Item()

    let now = DateTime.local()
    item.timestamp_ms_utc = now.valueOf()
    item.utc_offset_minutes = now.offset

    let comment = new Comment()
    item.comment = comment
    
    let ref = new ReplyRef()
    ref.user_id = new ProtoUserID()
    ref.user_id.bytes = replyToUserID.bytes
    ref.signature = new ProtoSignature()
    ref.signature.bytes = replyToSignature.bytes
    // ref.item_type = // TODO

    comment.reply_to = ref
    comment.text = text

    return item
}()


</script>

<style>
.item :global(textarea) {
    margin-bottom: 1em;    
}

</style>