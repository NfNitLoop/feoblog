<!--
    A button meant to appear alongside text.
-->
<img {src} {alt} on:click={onClick}>

<script lang="ts">
import { createEventDispatcher } from "svelte";

export let src: string
export let alt: string

 let dispatcher = createEventDispatcher()

 function onClick() {
     dispatcher("click")
 }

</script>

<style>

img {
    height: 1.1em;
    transition: transform 300ms;
}

img:hover {
    cursor: pointer;
    transform: scale(1.1);
}

</style>