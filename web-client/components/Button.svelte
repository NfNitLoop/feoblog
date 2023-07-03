<div class="buttonPosition {$$props.class}">
    <div class="button" class:disabled on:mouseup={clicked}>
        <slot/>
    </div>
</div>

<script lang="ts">

/*
 * emits a "click" event
 */

import { createEventDispatcher } from "svelte";

export let disabled = false

// Optionally specify an href to make this act like a link.
// If it starts with #, then it's an internal link and we navigate there.
// Otherwise, we open up a new window (tab) to the new external URL.
export let href = ""

let dispatcher = createEventDispatcher()

function clicked(event: MouseEvent) {
    if (disabled) return

    // Only click on left clicks!
    if (event.button !== 0) {
        return
    }

    if (href) {
        if (href.startsWith("#")) {
            window.location.hash = href
        } else {
            window.open(href)
        }
        return
    }

    dispatcher("click")
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
    white-space: nowrap;
}
.button:hover {
    box-shadow: 0px 4px 8px rgba(0, 0, 0, 0.5);
}
.button:active, .button.disabled{
    box-shadow: none;
    background-color: #eee;
    cursor: default;
}
.button.disabled {
    color: #888;
    box-shadow: 0px 2px 4px rgba(0, 0, 0, 0.2);
}


/* Required so that we can use position: absolute on the confirmation box. */
.buttonPosition {
    position: relative;
}

@keyframes buttonConfirm {
    to{
        background-color: red;
        color: white;
    }
}

</style>
