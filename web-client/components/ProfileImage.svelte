<div class="pic" class:isIdenticon class:isPhoto {style}></div>


<script lang="ts">
import type { UserID } from "../ts/client";

import Identicon from "identicon.js"


// TODO: Eventually we'll want to look up user icons for people.
// export let appState: Writable<AppState>

export let userID: UserID
let isIdenticon = true
let isPhoto = false


let style = ""
$: style = getStyleFor(userID)

function getStyleFor(userID: UserID): string {
    let icon = new Identicon(userID.toHex(), {
        size: 100,
        format: 'svg',
        background: [255, 255, 255, 0],
    })

    return `background-image: url("data:image/svg+xml;base64,${icon}");`
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

.pic.isIdenticon {
    /* border: 1px solid rgba(0,0,0,0.5) */
    background-color: white;
}

.pic.isPhoto {
    background-color: #888;
}
</style>