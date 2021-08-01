<!--
    See: https://github.com/sveltejs/svelte/issues/5960  🤦‍♂️
-->
<svelte:options immutable/>

<script lang="ts">
// View of a single item.
import type { Writable } from "svelte/store"
import { push as navigateTo } from "svelte-spa-router"

import { UserID} from "../ts/client"
import { markdownToHtml, fixLinks, FileInfo} from "../ts/common"
import Button from "./Button.svelte"
import type { Item } from "../protos/feoblog"
import type { AppState } from "../ts/app"
import UserIdView from "./UserIDView.svelte"
import CommentView from "./CommentView.svelte"
import ItemHeader from "./ItemHeader.svelte"
import { createEventDispatcher } from "svelte";

export let userID: string
export let signature: string

// Caller can provide a pre-fetched Item. 
// DO NOT BIND. If you want to see the item loaded, use on:itemLoaded
let initialItem: Item|null|undefined // = undefined // weird, causes type errors in callers.
// TODO: types don't seem to work well w/ export aliases like this. Just change the names:
export {initialItem as item}


// The item that we loaded:
let item: Item|null|undefined = undefined


export let appState: Writable<AppState>

// TODO: Remove
export let showDetail = false

// Show information about what this is in reply to.
// Might want to hide if it's obvious from context.
export let showReplyTo = true

// How should we handle clicks on links in this item view?
// newWindow: All links open in a new window.
// fix: 
//     Fix any links that would unnecessarily navigate out of the client. 
//     ex: /u/x/ => #/u/x/
//     But leaves external links alone.
//   
export let linkMode: "fix" | "newWindow" | "ignore" = "fix"
// This is a preview of an in-progress Item:
// TODO: Maybe deprecate linkMode now that there's previewMode?
export let previewMode = false

// Can we click on the item body to go to its page?
export let clickable = false

// When in preview mode, caller can provide a list of file attachments
// which we'll use to preview files.
export let previewFiles: FileInfo[] = []

let loadError = ""
let viewMode: "normal"|"markdown"|"data" = "normal"

let dispatcher = createEventDispatcher()

$: getItem(userID, signature, initialItem)
async function getItem(userID: string, signature: string, initialItem: Item|null|undefined) {
    if (initialItem !== undefined) {
        item = initialItem
        return
    }

    try {
        let result = await $appState.client.getItem(userID, signature)
        item = result
    } catch (e) {
        loadError = `Error loading item: ${e}`
        console.error(e)
    }

    dispatcher("itemLoaded", item)
}

let validFollows: ValidFollow[] = []
$: validFollows = function(){
    if (!item?.profile?.follows) { return [] }
    let valid: ValidFollow[] = []
    for (let follow of item.profile.follows) {
        try {
            let id = UserID.fromBytes(follow.user.bytes)
            valid.push({
                userID: id,
                displayName: follow.display_name.trim() || id.toString(),
            })
        } catch (e) {
            console.warn(`Error parsing follow for ${userID}`, e)
        }
    }
    return valid
}()

class ValidFollow {
    userID: UserID
    displayName: string
}

function onClick(event: Event) {
    let target = event.target as HTMLElement
    let anchor: HTMLAnchorElement|undefined = undefined
    let tag = target.tagName

    if (tag == "A") {
        anchor = (target as HTMLAnchorElement)
    } else if (tag == "IMG") {
        let parent = target.parentElement
        if (parent?.tagName == "A") {
            anchor = (parent as HTMLAnchorElement)
        }
    }


    if (anchor) { 
        // The user clicked a link, don't navigate anywhere.
        return
    }

    // Else this is not a link, we want to just navigate to the item's individual page:
    if (clickable) {
        let selection = window.getSelection()
        // Don't count as a navigation click if user is selecting text:
        if (!selection || selection.isCollapsed) {
            navigateTo(`#/u/${userID}/i/${signature}`)
            return
        }
    }
}
</script>   

{#if item === undefined}
    <div class="item">
        <div class="body">
            <p>Loading...
                <!-- 
            <br>user_id: { userID }
            <br>signature: { signature }
            -->
            </p>
        </div>
    </div>
{:else if loadError}
    <div class="item">
        <div class="body">
            <p class="error">Error: {loadError}
        </div>
    </div>
{:else}<!-- item && !loadError-->
<div class="item" class:clickable class:comment={item?.comment} on:click={onClick} use:fixLinks={{mode: linkMode}}>
    {#if item === null}
        <div class="body">
            No such item: <code>/u/{userID}/i/{signature}/</code>
        </div>
    {:else if item.post}
        <ItemHeader {appState} {item} userID={UserID.fromString(userID)} {signature} {previewMode} bind:viewMode />
        <div class="body">
            {#if item.post.title}
                <h1 class="title">{ item.post.title }</h1>
            {/if}        
            {#if viewMode == "normal"}
                {@html markdownToHtml(item.post.body || "", {withPreview: previewFiles, relativeBase: `/u/${userID}/i/${signature}/`})}
            {:else if viewMode == "markdown"}
                Markdown source:
                <code><pre>{item.post.body}</pre></code>
            {:else} 
                JSON representation of Protobuf Item:
                <code><pre>{JSON.stringify(item.toObject(), null, 4)}</pre></code>
            {/if}

        </div>
    {:else if item.profile}
        <ItemHeader {appState} {item} userID={UserID.fromString(userID)} {signature} {previewMode} />
        <div class="body">
            <h1 class="title">Profile: {item.profile.display_name}</h1>
            <div class="userIDInfo">
                id: <UserIdView userID={UserID.fromString(userID)} resolve={false} shouldLink={false} />
            </div>
            {#if viewMode == "normal"}
                {@html markdownToHtml(item.profile.about)}
            {:else if viewMode == "markdown"}
                Markdown source:
                <code><pre>{item.profile.about}</pre></code>
            {:else} 
                JSON representation of Protobuf Item:
                <code><pre>{JSON.stringify(item.toObject(), null, 4)}</pre></code>
            {/if}

            <h2>Follows</h2>
            <ul>
            {#each validFollows as follow}
                <li><UserIdView userID={follow.userID} displayName={follow.displayName} resolve={false}/></li>
            {:else}
                <li>(None)</li>    
            {/each}
            </ul>

            <h2>Servers</h2>
            <ul>
                {#each item.profile.servers as server (server)}
                    <!-- NOT hyperlinking this for now, in case someone tries to inject a javascript: link. -->
                    <li><code>{server.url}</code></li>
                {:else}
                    <li>(None)</li>
                {/each}
            </ul>
        </div>
    {:else if item.comment}
        <CommentView {appState} {showReplyTo} {item} 
            userID={UserID.fromString(userID)}
            {signature}
        />
    {:else}
        Unknown item type.
    {/if}
</div>
{/if}


<style>
.clickable {
    cursor: pointer;
}

.userIDInfo {
    font-size: 0.8em;
}

.userIDInfo, .userIDInfo :global(.userID) {
    color: #888;
}

</style>