<textarea
    bind:this={element}
    bind:value
    class:medium
    class:small
    class:oneLine
    {disabled}
    {placeholder}
></textarea>


<script lang="ts">
export let value: string
export let disabled = false
export let placeholder = ""

export let size: "medium"|"small"|"oneLine" = "medium"

$: medium = size == "medium"
$: small = size == "small"
$: oneLine = size == "oneLine"

let element: HTMLElement

$: {
    value // on change:
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

<style> 

textarea.medium {
    min-height: 20em;
}

textarea.oneLine {
    height: 1em;
}


   
</style>