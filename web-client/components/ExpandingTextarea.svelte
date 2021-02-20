<textarea
    bind:this={textarea}
    bind:value
    class:medium
    class:small
    class:oneLine
    {disabled}
    {placeholder}
></textarea>


<script lang="ts">
import { createEventDispatcher, tick } from "svelte";


export let value: string
export let disabled = false
export let placeholder = ""

export let size: "medium"|"small"|"oneLine" = "medium"

let dispatcher = createEventDispatcher()

$: medium = size == "medium"
$: small = size == "small"
$: oneLine = size == "oneLine"

let textarea: HTMLTextAreaElement

$: {
    value // on change:
    expandTextarea(textarea)
}

function expandTextarea(textarea: HTMLElement) {
    if (!textarea) { return } // not mounted yet
    
    // Only grows:
    if (textarea.scrollHeight > textarea.clientHeight) {
        let borderHeight = textarea.offsetHeight - textarea.clientHeight
        textarea.style.height = (textarea.scrollHeight + borderHeight).toString()
    }
}

$: {
    if (textarea) {
        textarea.addEventListener("paste", onPaste)
    }
}

function onPaste(event: ClipboardEvent) {
    dispatcher("paste", event)
}

// Treating the text as Markdown, add a link to it, respecting the current position.
// Keeps link refs at the bottom of the text.
export async function addLink({text, href, asImage}: LinkOptions) {
    if (href.search(" ") >= 0) {
        href = `<${href}>`
    }

    let imagePrefix = asImage ? "!" : ""
    let link = `${imagePrefix}[${text}]`
    let ref = `[${text}]: ${href}`

    let cursor = textarea.selectionEnd
    let adjustCursor = 0
    let before = value.substr(0, cursor)
    let after = value.substr(cursor)

    let parts = [before]
    if (before.length > 0) {
        if (asImage) {
            if (!before.endsWith("\n")) {
                parts.push("\n")
                adjustCursor++
            }
        } else {
            if (!before.endsWith(" ")) {
                parts.push(" ")
                adjustCursor++
            }
        }
    }
   
    parts.push(link)
    adjustCursor += link.length
    if (asImage) {
        parts.push("\n")
        adjustCursor++
    }

    parts.push(after)
    if (after.length == 0 || !after.endsWith("\n")) {
        parts.push("\n")
    }
    parts.push(ref)

    value = parts.join("")

    // Must wait to render the new text before setting cursor in it:
    await tick()

    cursor += adjustCursor
    textarea.setSelectionRange(cursor, cursor)
}


</script>

<script lang="ts" context="module">
type LinkOptions = {
    text: string,
    href: string,
    asImage?: boolean
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