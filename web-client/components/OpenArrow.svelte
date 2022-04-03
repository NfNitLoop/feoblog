<!--
    The arrow points left, and rotates down when toggled to "open".
    Could add an option to point right if needed.
-->

{#if icon == "arrow"}
<svg on:click={onClick} class:isOpen class:left viewBox="0 0 200 200" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M67 34L133 100L67 166" stroke="black" stroke-width="13" stroke-linecap="round"/>
</svg>
{:else}
<svg on:click={onClick} class:isOpen class:lineHeight viewBox="0 0 200 200" fill="none" xmlns="http://www.w3.org/2000/svg">
    <g >
    <line x1="6.5" y1="-6.5" x2="153.5" y2="-6.5" transform="matrix(1 3.06077e-05 -3.81057e-05 1 20 57)" stroke="black" stroke-width="13" stroke-linecap="round"/>
    <line x1="6.5" y1="-6.5" x2="153.5" y2="-6.5" transform="matrix(1 3.18946e-05 -3.65682e-05 1 20 106.497)" stroke="black" stroke-width="13" stroke-linecap="round"/>
    <line x1="6.5" y1="-6.5" x2="153.5" y2="-6.5" transform="matrix(1 3.15589e-05 -3.69572e-05 1 20 155.995)" stroke="black" stroke-width="13" stroke-linecap="round"/>
    </g>
</svg>
{/if}    

<script lang="ts">

export let icon: "arrow"|"burger" = "arrow"
export let isOpen = false
export let face: "right" | "left" = "left"
export let size: "default"|"line-height" = "default"

$: left = face === "left"
$: lineHeight = size == "line-height"

function onClick() {
    isOpen = !isOpen
}
</script>

<style>
svg {
    height: 1.5em;
    transition: transform 200ms;
    user-select: none;
    cursor: pointer;
    z-index: 0;
}

.left {
    transform: rotate(180deg);
}

.isOpen {
    transform: rotate(90deg);
}

path, line {
    filter: drop-shadow(0px 5px 10px rgb(0,0,0,0.2));
    stroke: rgb(0,0,0,0.5);
}

/* Blocks the isOpen animation. 😢
svg:hover:not(.isOpen) {
    animation: 300ms infinite alternate wiggle;
}
@keyframes wiggle {
    from {
        transform: rotate(175deg);
    }
    to {
        transform: rotate(185deg);
    }
}
*/

svg:hover path, svg:hover line {
    stroke: black;
}

svg.lineHeight {
    height: 1.1em;
    text-align: top;
}

</style>