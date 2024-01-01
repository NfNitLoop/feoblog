<!--
    A button meant to appear alongside text.
-->
{#if icon == "search"}
<svg
   {width} {height}
   class:lineHeight
   on:click={onClick}
   viewBox="0 0 79.91243 79.912429"
   version="1.1">
  <title>{alt || icon}</title>
  <g
     id="layer1"
     transform="translate(-73.158659,-92.13154)"
     style="opacity:0.993">
    <circle
       style="fill:none;fill-opacity:1;stroke-width:7.5;stroke-miterlimit:4;stroke-dasharray:none;stroke-opacity:1"
       id="path833"
       cx="103.29967"
       cy="121.74126"
       r="21.782465" />
    <path
       style="fill:none;stroke-width:10;stroke-linecap:round;stroke-linejoin:miter;stroke-miterlimit:4;stroke-dasharray:none;stroke-opacity:1"
       d="m 122.9389,140.42005 16.58087,14.76"
       id="path837"
    />
  </g>
</svg>
{:else}<!-- home -->
<svg {width} {height} class:lineHeight viewBox="0 0 270 270" fill="none">
    <title>{alt || icon}</title>
    <mask id="path-1-inside-1_103_2" fill="white">
        <rect x="53" y="89" width="164" height="164" rx="23"/>
    </mask>
    <rect x="53" y="89" width="164" height="164" rx="23" stroke-width="55.6" stroke-linejoin="round" mask="url(#path-1-inside-1_103_2)"/>
    <path d="M134.911 17.192L24 117" stroke-width="27.8" stroke-linecap="round"/>
    <path d="M135 17L245.911 116.808" stroke-width="27.8" stroke-linecap="round"/>
</svg>
{/if}

<script lang="ts">
import { createEventDispatcher } from "svelte";

export let icon: "search" | "home"
export let alt = ""
export let width: string|null = null
export let height: string|null = null
$: lineHeight = !width && !height


 let dispatcher = createEventDispatcher()

 function onClick() {
     dispatcher("click")
 }

</script>

<style>
.lineHeight {
    height: 1.1em;
} 

svg {
    cursor: pointer;
}

svg circle, svg path, svg rect {
    stroke: #aaa;
    transition: stroke 500ms;
}

svg:hover circle, svg:hover path, svg:hover rect {
    stroke: black;
}
</style>