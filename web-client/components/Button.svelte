<div class="buttonPosition">
    <div class="button" class:disabled class:confirmationMode on:mouseup={clicked} on:mouseleave={onMouseLeave}>
        <slot/>
    </div>
    
    {#if requiresConfirmation}
    <div class="confirmation" class:confirmationMode>
        Confirm <slot/>
    </div>
    {/if}
</div>

<script lang="ts">

/*
 * emits a "click" event
 */

import { createEventDispatcher } from "svelte";

export let disabled = false
// Does this button require a second click?
export let requiresConfirmation = false

// Optionally specify an href to make this act like a link.
// If it starts with #, then it's an internal link and we navigate there.
// Otherwise, we open up a new window (tab) to the new external URL.
export let href = ""

// This button requires confirmation, and is currently asking for confirmation
let confirmationMode = false

const minClickDeltaMs = 300
let firstClick = 0

let dispatcher = createEventDispatcher()

function clicked(event: MouseEvent) {
    if (disabled) return

    // Only click on left clicks!
    if (event.button !== 0) {
        return
    }

    if (requiresConfirmation) {
        if (!confirmationMode) {
            confirmationMode = true
            firstClick = event.timeStamp
            return
        }
        // This is not our first click, check if the click was slow enough:
        let delta = event.timeStamp - firstClick
        if (delta < minClickDeltaMs) {
            // This click was too soon, could be accidental.
            return
        }
    }

    if (href) {
        if (href.startsWith("#")) {
            window.location.hash = href
        } else {
            window.open(href)
        }
        return
    }

    confirmationMode = false
    dispatcher("click")
}

function onMouseLeave() {
    confirmationMode = false
}

</script>

<style>
.buttonPosition {
    display: inline-block;
}

.button {
    border-radius: 6px;
    margin: 2px;
    padding: 0.2em 8px;
    display: inline-block;
    box-shadow: 0px 2px 4px rgba(0, 0, 0, 0.25);
    user-select: none;
    cursor: pointer;
    background-color: #fff;
}
.button:hover {
    box-shadow: 0px 4px 8px rgba(0, 0, 0, 0.5);
}
.button:active, .button.disabled{
    box-shadow: none;
    background-color: #eee;
}
.button.disabled {
    color: #888;
    box-shadow: 0px 2px 4px rgba(0, 0, 0, 0.2);
}

.button.confirmationMode {
    animation-name: buttonConfirm;
    animation-duration: 150ms;
    animation-iteration-count: 5;
    animation-fill-mode: forwards;
    animation-direction: alternate;
    animation-timing-function: ease-in-out;
}


/* Required so that we can use position: absolute on the confirmation box. */
.buttonPosition {
    position: relative;
}

.confirmation {
    display: none;

    border-radius: 3px;
    margin: 2px;
    padding: 3px 8px;
    box-shadow: 0px 2px 4px rgba(0, 0, 0, 0.5);
    user-select: none;
    background-color: #fff;
    pointer-events: none;
    position: absolute;
    /* Align the bottom of this button w/ the top of the enclosing div. */
    bottom: 100%;
}

.confirmation.confirmationMode {
    display: block;
    animation-name: swoopUp;
    animation-duration: 300ms;
    animation-fill-mode: forwards;
}

@keyframes swoopUp {
    from {
        bottom: 0%;
        opacity: 0;
    }
}

@keyframes buttonConfirm {
    to{
        background-color: red;
        color: white;
    }
}

</style>
