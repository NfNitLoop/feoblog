<div class="button" on:mouseup={clicked}>
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
        /* font-weight: bold; */
        /* color: #fff; */
        user-select: none;
        /* transition: box-shadow 50ms ease-in-out; */
        cursor: pointer;
    }
    .button:hover {
        box-shadow: 0px 4px 8px rgba(0, 0, 0, 0.5);
    }
    .button:active {
        /* transition: all 0ms; */
        box-shadow: none;
        background-color: #eee;
    }
</style>

<script lang="ts">
import { createEventDispatcher } from "svelte";

// TODO:
// export let disabled = false

/*
 * emits a "click" event
 */

let dispatcher = createEventDispatcher()

function clicked() {
    dispatcher("click")
}

</script>