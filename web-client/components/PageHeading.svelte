<!-- 
    PageHeading that may include a "settings" slot. 
    TODO: Can we remove the global pageHeading class and make this just be an .item? 

-->
<div class="pageHeading" class:atTop on:mouseenter={() => leftHandler.mouseEntered()} on:mouseleave={() => leftHandler.mouseLeft()}>

    <div class="top">
        <div class="inner">
            <slot></slot>
        </div>
        {#if hasSettings}
        <h1 class="settingsButton">
            <SVGButton src="/client/images/magnifying_glass.svg" alt="search" on:click={toggleSettings} />
        </h1>
        {/if}
    </div>

    {#if hasSettings && !settingsHidden}
        <div class="settings" transition:slide>
            <slot name="settings"></slot>
        </div>
    {/if}
</div>

<div class="detection" bind:this={element}></div>

<svelte:window bind:scrollY bind:outerWidth/>

<script lang="ts">
import { onDestroy } from "svelte";
import { slide } from "svelte/transition";
import SVGButton from "./SVGButton.svelte";

let element: HTMLElement

let scrollY: number
let outerWidth: number

$: hasSettings = !!$$slots.settings
$: atTop = isAtTop(scrollY, outerWidth)

// We really only accept these numbers to trigger a refresh in case they changed:
function isAtTop(_scrollY: number, _outerWidth: number): boolean {
    if (!element) return false
    let rect = element.getBoundingClientRect()
    return rect.top <= 1
}


let leftHandler = new MouseLeftHandler(() => { settingsHidden = true })
onDestroy(() => {
    leftHandler.cleanup()
})

let settingsHidden = true

function toggleSettings() {
    settingsHidden = !settingsHidden
}

</script>

<script lang="ts" context="module">

/**
 * Perform some action after the mouse has been gone for some time.
 */
class MouseLeftHandler {
    constructor(private callback: () => void) {}
    public delayMS = 5000

    private timer: number|null = null

    mouseEntered() {
        this.cleanup()
    }

    cleanup() {
        if (this.timer) {
            clearTimeout(this.timer)
            this.timer = null
        }
    }

    mouseLeft() {
        if (this.timer) {
            // Weird, already a timer running?
            return
        }

        this.timer = setTimeout(() => {this.timerDone()}, this.delayMS)
    }

    private timerDone() {
        const callback = this.callback
        try {
            callback()
        } catch  (error) {
            console.error("Error in MouseLeftHandler.callback:", error)
        }
    }

}

</script>


<style>
.pageHeading {
    margin: 1rem;
	position: sticky;
    top: 0px;
    transition: all 300ms;
    max-width: 55rem;
    /* Required so that transform'd items don't bleed through. Weird. */
    z-index: 1;
}

.pageHeading :global(h1) {
	margin-top: 0px;
	margin-bottom: 0px;
    transition: all 300ms;
}

.pageHeading.atTop {
    padding: 0.5rem 1.3rem;
    border-top-left-radius: 0px;
    border-top-right-radius: 0px;
}

.pageHeading.atTop :global(h1) {
    font-size: 1rem;
}

.settings {
    padding-top: 1rem;
}

.pageHeading.atTop {
    margin-left: 0;
    margin-right: 0;
    border-radius: 0;
}

/* Must use same @media selector as in style.css: */
@media(min-width: 55em) {
    .pageHeading.atTop {
        margin: 0.5rem;
        max-width: 56rem;
        border-radius: 0 0 20px 20px;
    }
}

.top {
    display: flex;
    align-items: center;
    justify-content: space-between;
}



</style>