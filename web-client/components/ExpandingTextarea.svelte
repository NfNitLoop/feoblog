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

    // First, find if there is a section of "link references" at the end:
    let [mainText, linkRefs] = getLinkRefs(value)
    
    let cursor = textarea.selectionEnd
    if (cursor > mainText.length) { 
        // The cursor was (probably accidentally) inside of the linkRefs section. Move it back:
        cursor = mainText.length
    }
    // TODO: We could also try to reposition the cursor so that it's not in the middle of a [link](or://url)
    
    let before = mainText.substr(0, cursor)
    let after = mainText.substr(cursor)

    let parts = [before]
    if (before.length > 0) {
        let pad = asImage ? endPad(before, "\n", 2) : endPad(before, " ", 1)
        parts.push(pad)
        cursor += pad.length
    }
   
    parts.push(link)
    cursor += link.length

    if (after.length > 0) {
        if (asImage) {
            if (!after.startsWith("\n")) parts.push("\n")
        } else {
            if (!after.startsWith(" ")) parts.push(" ")
        }
    }

    parts.push(after)

    mainText = parts.join("")
    parts = [mainText]

    // Separate the refLinks with \n\n:
    parts.push(endPad(mainText, "\n", 2))

    if (linkRefs.length > 0) {
        parts.push(linkRefs)
        parts.push("\n")
    }
    parts.push(ref)

    value = parts.join("")

    // Must wait to render the new text before setting cursor in it:
    await tick()

    // We've modified the cursor position, set it:
    textarea.setSelectionRange(cursor, cursor)
}

// Return the padding to add to the end of `text` such that it will
// be padded with at least `number` of pad.
function endPad(text: string, pad: string, times: number): string {
    let matchCount = times
    for (let matchCount = times; matchCount > 0; matchCount--) {
        if (text.endsWith(pad.repeat(matchCount))) {
            let needed = times - matchCount
            return pad.repeat(needed)
        }
    }

    // We found no matches, so pad the full amount:
    return pad.repeat(times)
}


const linkRefPat = /^\[[^\]]+\]: /
const whitespaceLine = /^\s*$/

function last<T>(items: T[]): T {
    return items[items.length - 1]
}

// Return [mainText, linkRefsText] by splitting out a chunk of linkRefs from
// the bottom of markdown text, if it exists.
function getLinkRefs(inputText: string): [string, string] {

    let lines = inputText.split("\n")

    // Remove trailing whitespace. Can be accidental, or maybe the user
    // added it to expand the text box.
    popWhitespace(lines)

    let linkRefs = []
    while (lines.length > 0 && last(lines).match(linkRefPat)) {
        linkRefs.push(lines.pop())
    }
    linkRefs.reverse() // we reversed order by popping from the bottom, so fix it.


    if (linkRefs.length == 0) {
        // We didn't find any linkrefs, return unmodified text.
        return [inputText, ""]
    }

    // Pop any remaining whitespace between the main body and the ending linkRefs section.
    // (We'll re-add it when we add the new linkRef)
    // popWhitespace(lines)

    return [lines.join("\n"), linkRefs.join("\n")]
}

function popWhitespace(lines: string[]) {
    while (lines.length > 0 && last(lines).match(whitespaceLine)) {
        lines.pop()
    }
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