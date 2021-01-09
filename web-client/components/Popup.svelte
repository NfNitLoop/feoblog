<div class="item popup" bind:this={element} >
    <div class="titlebar" on:mousedown={onMouseDown}>
        <button class="close" on:click={hide}>X</button>
        {#if title && false}<h1>{title}</h1>{/if}
    </div>
    
    <slot/>
</div>

<script lang="ts">
export let shown = false
export let title = ""

export function show(event) {
    element.style.setProperty("visibility", "visible")
    left = event.x
    top = event.y

    shown = true
}
export function hide() {
    shown = false
    element.style.removeProperty("visibility")

}
export function toggle(event) {
    if (shown) { hide() }
    else { show(event) }
}

$: {
    if (element) {
        element.style.setProperty("left", left as any)
        element.style.setProperty("top", top as any)
    }
}

let element: HTMLElement

let top = 0
let left = 0
let moving = false
function onMouseDown(event) {
    moving = true
}

function onMouseUp() {
    moving = false
}

function onMouseMove(event) {
    if (!moving) { return }
    left += event.movementX
    top += event.movementY
}

</script>

<svelte:window on:mouseup={onMouseUp} on:mousemove={onMouseMove}/>

<style>
.popup {
    position: absolute;
    visibility: hidden;
    margin: 0px;
    margin-left: 1em;
}

.titlebar h1 {
    margin-top: 0px;
    margin-bottom: 0px;
    user-select: none;
}

.titlebar {
    margin-top: -0.8em;
    font-size: 0.8em;
    border-bottom: 1px solid black;
    margin-bottom: 0.5em;
}

button.close {
    float: right;
    padding: 2px;
}
</style>