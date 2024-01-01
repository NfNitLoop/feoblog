<!--
    Handle safely toggling visibility for an element at the top of the page. to account for its scroll position change.
-->

{#if shown}
    <slot></slot>
{/if}
<div bind:this={theDiv}></div>

<script lang="ts">
import { tick } from "svelte";
import { ConsoleLogger, scrollState } from "../../ts/common";

export let condition: boolean
let theDiv: HTMLElement

// private/internal:
let shown = condition
let promise: Promise<unknown>|undefined
const log = new ConsoleLogger({prefix: "ShowWhen"}) //.withDebug()

$: onChange(condition)
function onChange(newValue: boolean) {
    if (promise) { 
        // already waiting to apply state change, no need to do it again.
        return
    }
    promise = scrollState.withQuietLock(async () => {
        log.debug("toggle shown from", shown, "to", condition)
        shown = condition
        await tick()
        promise = undefined
        return isAbove()
    })
}

// Is the item above the current viewport?
function isAbove(): boolean {
    return theDiv.getBoundingClientRect().bottom < 0
}

</script>