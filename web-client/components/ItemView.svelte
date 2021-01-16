<script lang="ts">
// View of a single item.
import type { Writable } from "svelte/store"

import { UserID as ClientUserID} from "../ts/client"
import { markdownToHtml } from "../ts/common"
import Timestamp from "./Timestamp.svelte"
import Button from "./Button.svelte"
import type { Item } from "../protos/feoblog"
import type { AppState } from "../ts/app"
import UserIdView from "./UserIDView.svelte"

export let userID: string
export let signature: string
// Caller can provide a pre-fetched Item
export let item: Item|null|undefined = undefined
export let appState: Writable<AppState>
export let showDetail = false

// How should we handle clicks on links in this item view?
// newWindow: All links open in a new window.
// fix: 
//     Fix any links that would unnecessarily navigate out of the client. 
//     ex: /u/x/ => #/u/x/
//     But leaves external links alone.
//   
export let linkMode: "fix" | "newWindow" | "ignore" = "fix"


let itemPromise: Promise<Item|null>

// Support routing from svelte-spa-router.
// 🙁 See: https://github.com/ItalyPaleAle/svelte-spa-router/issues/183
export let params: any|undefined
$: {
    if (params) {
        userID = params.userID
        signature = params.signature
   }
} 

let viewMode: "normal"|"markdown"|"data" = "normal"


$: {
    // Rerun getItem when any of these change:
    itemPromise = getItem(userID, signature); item
}

async function getItem(userID: string, signature: string) {
    if (item !== undefined) {
        // User has provided their own, don't load one:
        return item
    }
    return await $appState.client.getItem(userID, signature)
}

function createComment() {
    console.log("TODO: createComment() unimplemented")
}

let validFollows: ValidFollow[] = []
$: validFollows = function(){
    if (!item?.profile?.follows) { return [] }
    let valid: ValidFollow[] = []
    for (let follow of item.profile.follows) {
        try {
            let id = ClientUserID.fromBytes(follow.user.bytes)
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
    userID: ClientUserID
    displayName: string
}

function interceptLinkClicks(event: Event) {

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

    if (!anchor) { return }

    if (linkMode === "ignore") {
        event.preventDefault()
        console.log("linkMode==ignore, ignored click to", anchor)
        return
    }

    // Note: can't use anchor.href, because that gets resolved to a full http://blah.com/wharrgarbl.
    // We want to know if this is a relative link.
    let href = anchor.getAttribute("href")
    if (!href) return

    let isRelative = href.startsWith("/") && !href.startsWith("//")
    let isAppLink = href.startsWith("#/")


    if (isRelative) {
        anchor.href = `#${href}`
        return
    }

    if (linkMode === "newWindow") {
        anchor.target = "_blank"
    }
}

</script>   


<div class="item" on:click={interceptLinkClicks}>
{#await itemPromise}
    <p>Loading...
        <!-- 
    <br>user_id: { userID }
    <br>signature: { signature }
    -->
    </p>
{:then item}
        {#if !item}
            No such item: <code>/u/{userID}/i/{signature}/</code>
        {:else if item.post}
            {#if item.post.title}
            <h1 class="title">{ item.post.title }</h1>
            {/if}
            <div class="userInfo">
                <UserIdView userID={ClientUserID.fromString(userID)} {appState}/>
            </div>
            <Timestamp utc_ms={item.timestamp_ms_utc} minute_offset={item.utc_offset_minutes} href={`#/u/${userID}/i/${signature}/`} />
            
            {#if viewMode == "normal"}
                {@html markdownToHtml(item.post.body || "")}
            {:else if viewMode == "markdown"}
                Markdown source:
                <code><pre>{item.post.body}</pre></code>
            {:else} 
                JSON representation of Protobuf Item:
                <code><pre>{JSON.stringify(item.toObject(), null, 4)}</pre></code>
            {/if}

            {#if showDetail}
            <div>
                <Button on:click={createComment}>Comment</Button>
                {#if viewMode != "normal"}<Button on:click={() => viewMode = "normal"}>View Normal</Button>{/if}
                {#if viewMode != "markdown"}<Button on:click={() => viewMode = "markdown"}>View Markdown</Button>{/if}
                {#if viewMode != "data"}<Button on:click={() => viewMode = "data"}>View Data</Button>{/if}
            </div>
            {/if}
        {:else if item.profile}
            <h1 class="title">Profile: {item.profile.display_name}</h1>
            <div class="userInfo">
                <UserIdView userID={ClientUserID.fromString(userID)} resolve={false}/>
            </div>
            <Timestamp utc_ms={item.timestamp_ms_utc} minute_offset={item.utc_offset_minutes} />

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
            
        {:else}
            Unknown item type.
        {/if}
{:catch error}
    <p>Error: {error}
{/await} 
</div>
