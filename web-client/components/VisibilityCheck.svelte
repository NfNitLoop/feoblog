<!--
    Component that just checks whether it's visibile on the page.
    emits event: itemVisible, when the item becomes visible
-->
<div bind:this={element}></div>

<script lang="ts">
import { createEventDispatcher, onDestroy, onMount } from "svelte";

// Always starts false so that we emit at least one itemVisible event if the item is visible.
export let visible: boolean = false

let element: HTMLElement
let observer = new IntersectionObserver(observerCallback)
let dispatch = createEventDispatcher()

onMount(() => {
    observer.observe(element)
})

onDestroy(() => {
    observer.disconnect()
})

function observerCallback(entries: IntersectionObserverEntry[], observer: IntersectionObserver) {
    // We only observe this one element, so this should always be here:
    let entry = entries[0]
    let nowVisible = entry.isIntersecting

    if (visible === nowVisible) {
        return // Nothing to do.
    }

    // Always change the current visibility before emitting event, or we're sending mixed singals
    visible = nowVisible

    if (nowVisible) {
        dispatch("itemVisible")
    }
}

</script>

<style>
div {
    /* Need some height, or the element can scroll out of view.*/
    min-height: 200px;
}
</style>