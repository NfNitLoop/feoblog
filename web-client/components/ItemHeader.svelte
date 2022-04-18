<!--
    The .header inside of an .item
-->
<div class="header">
    <ProfileImage {userID} />
    <div class="text">
        <UserIdView {userID} />
        {#if showReplyTo && item.comment != null}
            <a href={refToLink(item.comment.reply_to)}>
            replied to
            <UserIdView 
                userID={UserID.fromBytes(item.comment.reply_to.user_id.bytes)}
                shouldLink={false}
            />
            </a>
        {:else if item.profile} 
            <span>updated their profile</span>
        {/if}
        <Timestamp utc_ms={item.timestamp_ms_utc} minute_offset={item.utc_offset_minutes} href={timestampLink} showRelative={!previewMode} />
    </div>
    <OpenArrow bind:isOpen={arrowOpen} />
</div>

{#if arrowOpen}
<item-extras class="inset" class:hasTabs={viewMode} transition:slide|local>
    {#if shareURL && feoBlogURL}
    <share-links>
        <link-label>Share To</link-label>
        <other-sites>
            <Button href={tweetURL}>Twitter</Button>
            <Button href={facebookURL}>Facebook</Button>
            <Button href={redditURL}>Reddit</Button>
            
        </other-sites>
        <link-label>Full URL</link-label>
        <CopyBox value={shareURL} />

        <link-label>Relative URL</link-label>
        <CopyBox value={feoBlogURL} />
    </share-links>
    {/if}

    {#if viewMode}
    <view-tabs>
        {#each renderModes as [mode, name] (name)}
            <view-tab class:active={viewMode == mode} on:click={() => {viewMode=mode}} >{name}</view-tab>
        {/each}
    </view-tabs>
    {/if}

</item-extras>
{/if}

<svelte:options immutable/>

<script lang="ts" context="module">
export type ViewMode = "normal"|"markdown"|"data"
</script>

<script lang="ts">
import { getContext } from "svelte";
import type { Writable } from "svelte/store";
import { slide } from "svelte/transition"
import type { Item, ReplyRef } from "../protos/feoblog";
import type { AppState } from "../ts/app";
import { Signature, UserID } from "../ts/client";

import UserIdView from "./UserIDView.svelte"
import Timestamp from "./Timestamp.svelte"
import ProfileImage from "./ProfileImage.svelte";
import OpenArrow from "./OpenArrow.svelte";
import CopyBox from "./CopyBox.svelte";
import Button from "./Button.svelte";

let appState: Writable<AppState> = getContext("appStateStore")

// required:
export let userID: UserID
export let signature: string|undefined // could be invalid/missing during preview
export let item: Item
export let previewMode = false

// optional:
export let showReplyTo = true

export let viewMode: ViewMode |undefined = undefined

// Readable:
export let arrowOpen = false


const renderModes: [mode: ViewMode, name:string][]  = [
    ["normal", "Rendered"],
    ["markdown", "Markdown"],
    ["data", "Protobuf"],
]

function refToLink(ref: ReplyRef): string {
    let uid = UserID.fromBytes(ref.user_id.bytes)
    let sig = Signature.fromBytes(ref.signature.bytes)
    return `#/u/${uid}/i/${sig}/`
}

$: timestampLink = parsedSignature ? `#/u/${userID}/i/${signature}/` : undefined
$: parsedSignature = function() {
    if (!signature) { return null }
    try {
        return Signature.fromString(signature)
    } catch (error) {
        console.error("Could not parse signature:", signature)
    }
}()

$: feoBlogURL = !parsedSignature ? undefined : `/u/${userID}/i/${signature}/`
$: shareURL = !parsedSignature ? undefined : `${window.location.protocol}//${window.location.host}${feoBlogURL}`

// See: https://developer.twitter.com/en/docs/twitter-for-websites/tweet-button/overview
$: tweetURL = function() {
    if (!shareURL) return ""

    const url = new URL("https://twitter.com/intent/tweet")
    url.searchParams.set("text", `\n\n${shareURL}`)
    return url.toString()
}()

// See: https://www.facebook.com/sharer/sharer.php
// See: https://stackoverflow.com/questions/9120539/facebook-share-link-without-javascript
$: facebookURL = function() {
    if (!shareURL) return ""

    const url = new URL("https://www.facebook.com/sharer/sharer.php")
    url.searchParams.set("u", shareURL)
    return url.toString()
}()

// See: https://stackoverflow.com/questions/24823114/post-to-reddit-via-url
$: redditURL = function() {
    if (!shareURL) return ""

    const url = new URL("http://www.reddit.com/submit")
    url.searchParams.set("url", shareURL)
    // TODO: title if available.
    return url.toString()
}()
</script>

<style>


item-extras.hasTabs {
    padding-bottom: 0px;
    margin-bottom: 0px;
}



share-links {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: 0.5rem;
}

link-label {
    display: block;
    grid-column-start: 1;
    text-align: right;
    font-weight: bold;
    flex-shrink: 1;
}

link-label::after {
    content: ":";
}

other-sites {
    text-align: right;
}

view-tabs {
    display: block;
    margin: 1em 1em 0em 0em;
    --tab-radius: 5px;
    padding-bottom: 0px;
}

view-tab {
    /* border: 1px solid black; */
    display: inline-block;
    border-bottom: 0px;
    border-radius: var(--tab-radius) var(--tab-radius) 0px 0px;
    background: #ddd;
    padding: 0.5em 0.8em;
    margin: 0 0.3em;
    color: black;
    cursor: pointer;
    user-select: none;
    font-weight: bold;
}

.active {
    background: white;
}



</style>
