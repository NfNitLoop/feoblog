<!-- 
    Displays info about file attachments 
    Provides an API for common file-attachment operations.
    Makes the entire window onDragOver and onDrop, so can only be used once per page.
-->

<h2>Attachments</h2>

{#if hasFiles}
<div class="inputsGreyBox">
<table class="attachments">
    {#each files as file (file)}
    <tr>
        <td>
        {#if file.isImage} 
        <img alt={file.name} src={file.objectURL}>
        {/if}
        </td>
        <td>{file.name}</td>
        <td>{file.readableSize}</td>
        <td><Button on:click={() => removeFile(file)}>Remove</Button></td>
    </tr>
    {/each}
</table>
</div>
{/if}

<input type="file" multiple bind:this={fileInput} on:change={onFileAttached}>
<Button on:click={() => fileInput.click()} disabled={!buttonsEnabled}>Attach File</Button>

<svelte:window on:dragover={onDragOver} on:drop={onDrop}/>

<script lang="ts">
import { createEventDispatcher } from "svelte"
import {FileInfo} from "../ts/common"
import Button from "./Button.svelte"


export let files: FileInfo[] = []
$: hasFiles = files.length  > 0

let fileInput: HTMLInputElement
let buttonsEnabled = true
let dispatcher = createEventDispatcher()

function onDragOver(e: DragEvent) {
    e.preventDefault()
    e.dataTransfer!.dropEffect = 'copy'
}

async function onDrop(e: DragEvent) {
    e.preventDefault()

    if (!e.dataTransfer) return

    let items = e.dataTransfer.items
    try {
        buttonsEnabled = false

        // Fix for a weird bug in Chrome:
        // If I call addFile() directly on the item while I'm iterating a DataTransferItemsList, 
        // THE LIST SEEMS TO BE MODIFIED.
        // Looks like it happens when I call File.arrayBuffer() over in FileInfo.from().
        // But we can work around it by just getting all file references out of the DataTransferItemsList
        // before calling arrayBuffer() on any of them.
        let files: File[] = new Array()
        for (let i = 0; i < items.length; i++) {
            let item = items[i].getAsFile()
            if (!item) continue
            files.push(item)
        }

        for (let file of files) {
            addFile(file)
        }

    } finally {
        buttonsEnabled = true
    }

}

// TODO: Is there a simpler type for e here?
// See: https://github.com/sveltejs/language-tools/blob/master/packages/svelte2tsx/svelte-jsx.d.ts
async function onFileAttached(e: Event & { currentTarget: HTMLInputElement}) {
    let inputFiles = e.currentTarget.files

    if (inputFiles === null) {
        console.warn("files list is null")
        return
    }

    await addFiles(inputFiles)

    // Note: we dont' have to clear inputFiles after we use it.
    // If the user clicks the button again, we get a new file list each time.
}

async function addFiles(files: FileList) {
    try {
        buttonsEnabled = false
        for (let i = 0; i < files.length; i++) {
            let f = files.item(i)
            if (f === null) {
                console.warn(`file list item # ${i} was null`)
                continue
            }
            await addFile(f)
        }
    } finally { 
        buttonsEnabled = true
    }
}

export async function addFile(file: File) {
    let fi = await FileInfo.from(file)

    let existingNames = new Set<string>()
    for (let existing of files) {
        if (existing.hash.asHex === fi.hash.asHex) {
            console.warn(`File "${fi.name}" (hash: ${fi.hash.asHex}) is already attached as "${existing.name}". Not adding again.`)
            return
        }

        existingNames.add(existing.name)
    }

    // Add .#.ext if this file name already exists.
    if (existingNames.has(fi.name)) {
        let match = fi.name.match(/[.][^.]+$/) 
        let extension = match ? match[0] : ""
        let base = fi.name.slice(0, fi.name.length - extension.length)

        // Start at 2, the "base" name is an implicit 1.
        let count = 2
        let newName;
        while (true) {
            newName = `${base}.${count}${extension}`
            if (!existingNames.has(newName)) break
            count++
        }
        fi.name = newName
    }

    files = [...files, fi]
    dispatcher("fileAdded", fi)
}

function removeFile(file: FileInfo) {
    files = files.filter((f) => { 
        const keep = (f !== file)
        if (!keep) {
            f.close()
        }
        return keep
     })
}


// TODO: revokeObjectURL()
// See: https://developer.mozilla.org/en-US/docs/Web/API/URL/createObjectURL


</script>


<style>
.attachments img {
    margin: 0.2em;
    border: 1px solid black;
    max-height: 3em;
    max-width: 6em;
    display: inline-block;
}

h2 {
    margin-top: 0;
}

table {
    width: 100%;
    border-spacing: 0px;
}

tr {
    padding-left: 0.2rem;
    padding-right: 0.2rem;
}

tr td {
    width: 1%;
}

tr td:nth-child(1) {
    text-align: right;
    padding-left: 0.5rem;
}

tr td:nth-child(2) {
    width: 80%;
}

tr td:nth-child(3) {
    white-space: nowrap;
}

tr td:nth-child(4) {
    padding-left: 0.5rem;
    padding: 0.5rem;
}
tr:hover{
    background-color: white;
}

.inputsGreyBox {
    padding-left: 0;
    padding-right: 0;
}

input[type="file"] {
    display: none;
}


</style>