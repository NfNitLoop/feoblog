<!--
    The arrow points left, and rotates down when toggled to "open".
    Could add an option to point right if needed.
-->

<svg on:click={onClick} class:isOpen class:left viewBox="0 0 200 200" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M67 34L133 100L67 166" stroke="black" stroke-width="13" stroke-linecap="round"/>
</svg>

<script lang="ts">

export let isOpen = false
export let face: "right" | "left" = "left"
$: left = face === "left"

function onClick() {
    isOpen = !isOpen
}
</script>

<style>
svg {
    height: 1.5em;
    transition: all 200ms;
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

path {
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

svg:hover path {
    stroke: black;
}

</style>