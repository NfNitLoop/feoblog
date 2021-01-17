<textarea bind:this={element} bind:value {disabled} {placeholder}></textarea>


<script lang="ts">
export let value: string
export let disabled = false
export let placeholder = ""

let element: HTMLElement

$: {
    value // on change:
    // TODO: Make a reusable expanding textarea widget.
    expandTextarea(element)
}

function expandTextarea(textarea: HTMLElement) {
    if (!textarea) { return } // not mounted yet
    
    // Only grows:
    if (textarea.scrollHeight > textarea.clientHeight) {
        let borderHeight = textarea.offsetHeight - textarea.clientHeight
        textarea.style.height = (textarea.scrollHeight + borderHeight).toString()
    }
}

</script>