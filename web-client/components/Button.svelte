<div class="button" class:disabled={disabled} on:mouseup={clicked}>
    <slot/>
</div>

<style>
    .button {
        border-radius: 3px;
        /* background: #eee; */
        margin: 2px;
        padding: 3px 8px;
        display: inline-block;
        box-shadow: 0px 2px 4px rgba(0, 0, 0, 0.5);
        user-select: none;
        /* transition: box-shadow 50ms ease-in-out; */
        cursor: pointer;
        background-color: #fff;
    }
    .button:hover {
        box-shadow: 0px 4px 8px rgba(0, 0, 0, 0.5);
    }
    .button:active, .button.disabled{
        /* transition: all 0ms; */
        box-shadow: none;
        background-color: #eee;
    }
    .button.disabled {
        color: #888;
        box-shadow: 0px 2px 4px rgba(0, 0, 0, 0.2);
    }
</style>

<script lang="ts">
import { createEventDispatcher } from "svelte";

export let disabled = false

/*
 * emits a "click" event
 */

let dispatcher = createEventDispatcher()

function clicked(event: MouseEvent) {
    // Only click on left clicks!
    if (event.button !== 0) {
        return
    }

    if (!disabled) {
        dispatcher("click")
    }
}

</script>