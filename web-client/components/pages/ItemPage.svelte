<PageHeading/>
{#if !userID || !signature}
<div class="error">
    Username and Signature are required to render this view.
</div>
{:else}
    <ItemView 
        userID={userID.toString()}
        signature={signature.toString()}
        on:itemLoaded={itemLoaded}
    />

    {#if allowComments}
    <div class="commentSection">
        <CommentEditor 
            replyToUserID={userID}
            replyToSignature={signature}
        />

        {#each replies as reply (reply)}
            <ItemView
                userID={reply.userID.toString()}
                signature={reply.signature.toString()}
                item={reply.item}
                showReplyTo={false}
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
{/if}

<script lang="ts">
import type { Writable } from "svelte/store"
import type { AppState } from "../../ts/app"
import type { Item } from "../../protos/feoblog";

import { getContext } from "svelte";
import { params } from "svelte-hash-router"

import { Signature, UserID } from "../../ts/client"

import ItemView from "../ItemView.svelte"
import CommentEditor from "../CommentEditor.svelte"
import Button from "../Button.svelte"
import PageHeading from "../PageHeading.svelte";

let appState: Writable<AppState> = getContext("appStateStore")

$: userID = UserID.tryFromString($params.userID)
$: signature = Signature.tryFromString($params.signature)
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


// Loads all comments for now. Will add pagination & filtering later.
async function loadComments(allowComments: boolean, userID: UserID|null, signature: Signature|null) {

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