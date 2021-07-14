<div class="pageHeading" class:scrolledAway on:mouseenter={onMouseEnter} on:mouseleave={onMouseLeave}>
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

<svelte:window bind:scrollY/>

<script lang="ts">
import { slide } from "svelte/transition";
import Button from "./Button.svelte"

let scrollY: number
$: scrolledAway = scrollY > 30
$: hasSettings = !!$$slots.settings

let mouseInside = false
let settingsHidden = true

function onMouseEnter() {
    mouseInside = true
}

function onMouseLeave() {
    mouseInside = false
    settingsHidden = true
}

function toggleSettings() {
    settingsHidden = !settingsHidden
}

</script>


<style>
    .pageHeading {
	top: 0px;
	position: sticky;
}

.pageHeading :global(h1) {
	margin-top: 0px;
	margin-bottom: 0px;
    transition: all 300ms;
}

.pageHeading.scrolledAway {
    padding: 0.8rem;
    padding-left: 1.3rem;
}

.pageHeading.scrolledAway :global(h1) {
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