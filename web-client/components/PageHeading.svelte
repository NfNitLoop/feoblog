<!-- 
    PageHeading that may include a "settings" slot. 
    TODO: Can we remove the global pageHeading class and make this just be an .item? 

-->
<div class="pageHeading" class:atTop bind:this={element} on:mouseenter={onMouseEnter} on:mouseleave={onMouseLeave}>
    {#if hasSettings}
    <div class="settingsButton">
        <Button on:click={toggleSettings}>Filter</Button>
    </div>
    {/if}

    <slot></slot>

    {#if hasSettings && !settingsHidden}
        <div class="settings" transition:slide>
            <slot name="settings"></slot>
        </div>
    {/if}
</div>

<svelte:window bind:scrollY bind:outerWidth/>

<script lang="ts">

import { slide } from "svelte/transition";
import Button from "./Button.svelte"

let element: HTMLElement

let scrollY: number
let outerWidth: number

$: hasSettings = !!$$slots.settings
$: atTop = isAtTop(scrollY, outerWidth)

// We really only accept these numbers to trigger a refresh in case they changed:
function isAtTop(_scrollY: number, _outerWidth: number): boolean {
    if (!element) return false;
    let rect = element.getBoundingClientRect()
    return rect.top === 0
}


let mouseInside = false
let settingsHidden = true

function onMouseEnter() {
    mouseInside = true
}

function onMouseLeave() {
    mouseInside = false
    // settingsHidden = true TODO: timer?
}

function toggleSettings() {
    settingsHidden = !settingsHidden
}

</script>


<style>
.pageHeading {
    margin: 1rem;
	position: sticky;
    top: 0px;
}

.pageHeading :global(h1) {
	margin-top: 0px;
	margin-bottom: 0px;
    transition: all 300ms;
}

.pageHeading.atTop {
    padding: 0.8rem;
    padding-left: 1.3rem;
    border-top-left-radius: 0px;
    border-top-right-radius: 0px;
}

.pageHeading.atTop :global(h1) {
    font-size: 1rem;
}

.settings {
    padding-top: 1rem;
}

.settingsButton {
    display: block;
    float: right;
}



</style>