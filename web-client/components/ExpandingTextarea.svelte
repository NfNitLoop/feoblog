<textarea
    bind:this={element}
    bind:value
    class:medium
    class:small
    {disabled}
    {placeholder}
></textarea>


<script lang="ts">
export let value: string
export let disabled = false
export let placeholder = ""

export let size: "medium"|"small" = "medium"

$: medium = size == "medium"
$: small = size == "small"

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
textarea {
    margin-top: 1em;
}

textarea.medium {
    min-height: 20em;
}


   
</style>