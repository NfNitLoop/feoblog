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
                    errors={errors}
                />
            {/if}
        </div>
    </div>
{:else}
    {#if userID != null}
    <!-- TODO: Just replace with ItemView: -->
    <div class="item">
        <CommentView
            item={commentItem}
            {userID}
            signature=""
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
import { getContext } from "svelte";
import type { Writable } from "svelte/store";
import type { AppState } from "../ts/app";
import type { Signature, UserID} from "../ts/client";
import { protobuf as pb} from "../ts/client";
import CommentView from "./CommentView.svelte";
import ExpandingTextarea from "./ExpandingTextarea.svelte";
import SignAndSend from "./SignAndSend.svelte";
import TabBar from "./TabBar.svelte";


let appState: Writable<AppState> = getContext("appStateStore")
export let replyToUserID: UserID
export let replyToSignature: Signature

let currentView: "Edit"|"Preview" = "Edit"

let text = ""
// Any time replyToSignature changes, we need to clear the text:
$: { replyToSignature; clear() }

function clear() {
    text = ""
}

$: userID = $appState.loggedInUser
$: isLoggedIn = userID != null
$: hasText = text.trim().length > 0
$: errors = !hasText ? ["Can not submit an empty comment"] : []
$: placeholder = isLoggedIn ? "Leave a Comment" : "Must log in to comment"

$: commentItem = function() {
    let item = new pb.Item()

    let now = DateTime.local()
    item.timestampMsUtc = BigInt(now.valueOf())
    item.utcOffsetMinutes = now.offset

    let comment = new pb.Comment()
    
    // Nope: item.comment = comment
    item.itemType = {case: "comment", value: comment}


    
    let ref = new pb.ReplyRef()
    ref.userId = new pb.UserID()
    ref.userId.bytes = replyToUserID.bytes
    ref.signature = new pb.Signature()
    ref.signature.bytes = replyToSignature.bytes
    // ref.item_type = // TODO

    comment.replyTo = ref
    comment.text = text

    return item
}()


</script>

<style>
.item :global(textarea) {
    margin-bottom: 1em;    
}

</style>