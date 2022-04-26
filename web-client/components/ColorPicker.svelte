<div>
    <input bind:this={inputter} class="color" type="color" bind:value={color}>
    <color-box bind:this={box} on:click={() => inputter.click()}></color-box>    
</div>


<script lang="ts">
import { createEventDispatcher } from "svelte";


export let color: string|null = null

let box: HTMLElement
let inputter: HTMLElement

let dispatch = createEventDispatcher()

$: if (box && color) {
    box.style.backgroundColor = color
}

// input on:change only fires when the choice is finalized.
// Show changes while we drag:
$: color && dispatch("change")

</script>


<style>
input {
    visibility: hidden;
    position: absolute;
}

div {
    display: inline-block;
}

color-box, input, div {
    width: 2rem;
    height: 2rem;
}

color-box {
    background-color: var(--default-bg-color);
    display: inline-block;
    border-radius: 5px;
    padding: 0px;
    box-shadow: 0px 3px 3px rgba(0,0,0,0.15);
    cursor: pointer;
}


</style>