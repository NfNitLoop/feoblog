<!--
    See: https://github.com/sveltejs/svelte/issues/5960  🤦‍♂️
-->
<svelte:options immutable/>

<script lang="ts" context="module">

export interface PageEvent {
    signature: string,
    userID: string,
    item?: pb.Item|null
    element: HTMLElement|null
}

</script>

<script lang="ts">
// View of a single item.
import type { Writable } from "svelte/store"

import { UserID, protobuf as pb, getInner } from "../ts/client"
import { markdownToHtml, fixLinks, FileInfo, observable, scrollState, ConsoleLogger} from "../ts/common"
import type { AppState } from "../ts/app"
import UserIdView from "./UserIDView.svelte"
import CommentView from "./CommentView.svelte"
import ItemHeader from "./ItemHeader.svelte"
import { createEventDispatcher, getContext, tick } from "svelte";
import UserIdChip from "./UserIDChip.svelte";

export let userID: string
export let signature: string

// Caller can provide a pre-fetched Item. 
// DO NOT BIND. If you want to see the item loaded, use on:itemLoaded
export let item: pb.Item|null|undefined = undefined

// When rendering items above our current location in the page, loading images
// can cause the block length to change, and cause the page to scroll.
// We can set their height to be constant to avoid that until they scroll into view.
export let shrinkImages = false

// The item that we loaded:
let loadedItem: pb.Item|null|undefined = undefined


let appState: Writable<AppState> = getContext("appStateStore")

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

// This item is "active" w.r.t. keyboard navigation, so should have a different style:
export let active = false

let loadError = ""
let viewMode: "normal"|"markdown"|"data" = "normal"

let dispatcher = createEventDispatcher()

let itemElement: HTMLElement|null = null

const logger = new ConsoleLogger({prefix: `<ItemView> ${signature.substring(0,10)}`}) //.withDebug()

$: getItem(userID, signature, item)
async function getItem(userID: string, signature: string, initialItem: pb.Item|null|undefined) {
    if (initialItem !== undefined) {
        loadedItem = initialItem
        return
    }

    try {
        let result = await $appState.client.getItem(userID, signature)
        loadedItem = result
    } catch (e) {
        loadError = `Error loading item: ${e}`
        logger.error(e)
    }

    dispatcher("itemLoaded", loadedItem)
}

$: post = getInner(loadedItem, "post")
$: profile = getInner(loadedItem, "profile")

// If this is a Profile, which users does this profile follow?
let validFollows: ValidFollow[] = []
$: validFollows = function(){
    if (!loadedItem || loadedItem.itemType.case != "profile") return []
    let profile = loadedItem.itemType.value
    if (!profile.follows) { return [] }
    let valid: ValidFollow[] = []
    for (let follow of profile.follows) {
        try {
            let id = UserID.fromBytes(follow.user!.bytes)
            valid.push({
                userID: id,
                displayName: follow.displayName.trim() || id.toString(),
            })
        } catch (e) {
            logger.warn(`Error parsing follow for ${userID}`, e)
        }
    }
    return valid
}()

interface ValidFollow {
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
            window.location.hash = `#/u/${userID}/i/${signature}/`
            return
        }
    }
}

function pageEventDetail(): PageEvent {
    return {userID, signature, item, element: itemElement}
}

async function enteredPage() {
    logger.debug("enteredPage:")   
    if (shrinkImages) {
        // Handle our own unshrink BEFORE dispatching notice to other elements,
        // which might re-render us w/o shrink, causing a race condition.
        await scrollState.withQuietLock(restoreImages)
    }

    dispatcher("enteredPage", pageEventDetail())
}

$: logger.debug("shrinkImages:", shrinkImages)

// Note: Should be run within a global lock, to prevent document offset math from having
// race conditions.
async function restoreImages(): Promise<boolean> {
    logger.log("restoreImages")

    if (!itemElement) {
        logger.warn("restoreImages, but no itemElement!?")
        return false
    }

    if (shrinkImages == false) {
        logger.warn("shrinkImages already false!?")
        return false
    }

    shrinkImages = false
    await tick()
   
    let bounds = itemElement.getBoundingClientRect()
    logger.debug("bounds.top", bounds.top)
    let needsAdjust = bounds.top < 0
    logger.debug("restoreImages needsAdjust:", needsAdjust)
    return needsAdjust
}

function leftPage() {
    dispatcher("leftPage", pageEventDetail())
}

</script>   

{#if loadedItem === undefined}
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
<!-- svelte-ignore a11y-click-events-have-key-events -->
<div
    class="item"
    id={signature}
    class:clickable
    class:comment={loadedItem?.itemType.case == "comment"}
    class:shrinkImages
    class:active
    on:click={onClick}
    use:fixLinks={{mode: linkMode}}
    use:observable={{enteredPage, leftPage}}
    bind:this={itemElement}
    on:mouseenter
    on:mousemove
>
    {#if loadedItem === null}
        <div class="body">
            <p>No such item:
                <br><code>/u/{userID}/i/{signature}/</code>
            </p>
        </div>
    {:else if post}
        <ItemHeader item={loadedItem} userID={UserID.fromString(userID)} {signature} {previewMode} bind:viewMode />
        <div class="body">
            {#if viewMode == "normal"}
                {#if post.title}
                    <h1 class="title">{ post.title }</h1>
                {/if}

                {@html markdownToHtml(post.body || "", {withPreview: previewFiles, relativeBase: `/u/${userID}/i/${signature}/`})}
            {:else if viewMode == "markdown"}
                <p>Markdown source:</p>
                <code><pre>{post.body}</pre></code>
            {:else} 
                <p>JSON representation of Protobuf Item:</p>
                <code><pre>{JSON.stringify(loadedItem, null, 4)}</pre></code>
            {/if}

        </div>
    {:else if profile}
        <ItemHeader item={loadedItem} userID={UserID.fromString(userID)} {signature} {previewMode} bind:viewMode />
        <div class="body">
            {#if profile.displayName} 
                <h1>{profile.displayName}</h1>
            {/if}
            <div class="userIDInfo">
                id: <UserIdView userID={UserID.fromString(userID)} resolve={false} shouldLink={false} />
            </div>
            {#if viewMode == "markdown"}
                <p>Markdown source:</p>
                <code><pre>{profile.about}</pre></code>
            {:else if viewMode == "data"} 
                <p>JSON representation of Protobuf Item:</p>
                <code><pre>{JSON.stringify(loadedItem, null, 4)}</pre></code>
            {:else}
                {@html markdownToHtml(profile.about, {relativeBase: `/u/${userID}/i/${signature}`})}

                {#if validFollows.length > 0}
                    <h2>Follows</h2>
                    <user-follows>
                    {#each validFollows as follow}
                        <UserIdChip 
                            userID={follow.userID}
                            displayName={follow.displayName}
                            shouldLink={!previewMode}
                        />
                    {:else}
                        <div>(None)</div>
                    {/each}
                    </user-follows>
                {/if}
    
                {#if profile.servers.length > 0}
                    <h2>Servers</h2>
                    <ul>
                        {#each profile.servers as server (server)}
                            <!-- NOT hyperlinking this for now, in case someone tries to inject a javascript: link. -->
                            <li><code>{server.url}</code></li>
                        {:else}
                            <li>(None)</li>
                        {/each}
                    </ul>
                {/if}
            {/if}


        </div>
    {:else if loadedItem.itemType.case == "comment"}
        <CommentView {showReplyTo} item={loadedItem} 
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

h1:has(+ .userIDInfo) {
    margin-bottom: 0;
}

.shrinkImages .body :global(img) {
    height: 5px;
}


.item.active {
    box-shadow: 0px 5px 20px rgb(0 0 0 / 80%);
}

user-follows {
    display: flex;
    flex-wrap: wrap;
}

</style>