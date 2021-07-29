{#if iconSize}
<div 
    class="pic"
    class:isIdenticon
    class:isPhoto
    {style}
></div>
{:else}
<img
    class="pic"
    class:fitSize
    src={imgSrc}
    alt=""
/>
{/if}

<script lang="ts">
import type { UserID } from "../ts/client";

import Identicon from "identicon.js"


// TODO: Eventually we'll want to look up user icons for people.
// export let appState: Writable<AppState>

export let userID: UserID

export let size: "icon"|"fit" = "icon"

$: iconSize = size == "icon"
$: fitSize = size == "fit"

let isIdenticon = true
let isPhoto = false


let style = ""
$: style = `background-image: url("${imgSrc}");`
$: imgSrc = getImgSrc(userID)

// TODO: Use a factory for these to re-use icons data:
// TODO: Actually, just use the server-side icons instead, since they can be cached easily.
function getImgSrc(userID: UserID): string {
    let icon = new Identicon(userID.toHex(), {
        size: 100,
        format: 'svg',
        background: [255, 255, 255, 0],
    })
    return `data:image/svg+xml;base64,${icon}`
}

</script>

<style>
.pic {
    height: 2rem;
    width: 2rem;
    background-position: center center;
    background-repeat: no-repeat;
    background-size: cover;

    display: inline-block;
    margin: 0.2rem;
    margin-right: 0.5rem;

    flex-grow: 0;
    flex-shrink: 0;

    border-radius: 5px;
    background-color: white;

    box-shadow: 0px 3px 3px rgba(0,0,0,0.15);
}

.fitSize {
    width: 100%;
    height: auto;
}

.pic.isIdenticon {
    /* border: 1px solid rgba(0,0,0,0.5) */
    background-color: white;
}

.pic.isPhoto {
    background-color: #888;
}
</style>