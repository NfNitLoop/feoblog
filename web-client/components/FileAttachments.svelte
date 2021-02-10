<!-- 
    Displays info about file attachments 
    Provides an API for common file-attachment operations.
-->

{#if hasFiles}
<h2>Attachments</h2>

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
        <td><Button requiresConfirmation on:click={() => removeFile(file)}>Remove</Button></td>
    </tr>
    {/each}
</table>
</div>
{/if}

<svelte:window on:dragover={onDragOver} on:drop={onDrop}/>

<script lang="ts">
import { createEventDispatcher } from "svelte"
import {FileInfo} from "../ts/common"
import Button from "./Button.svelte"


export let files: FileInfo[] = []
$: hasFiles = files.length  > 0

let dispatcher = createEventDispatcher()



function onDragOver(e: DragEvent) {
    // e.stopPropagation()
    e.preventDefault()
    // console.log("onDragOver", e)
    e.dataTransfer!.dropEffect = 'copy'

}

async function onDrop(e: DragEvent) {
    e.preventDefault()

    if (!e.dataTransfer) return

    let items = e.dataTransfer.items
    for (let i = 0; i < items.length; i++) {
        let item = items[i].getAsFile()
        if (!item) continue

        await addFile(item)
    }

}

async function addFile(file: File) {
    let fi = await FileInfo.from(file)

    for (let existing of files) {
        if (existing.hash.asHex === fi.hash.asHex) {
            console.warn(`File "${fi.name}" is already attached. Not adding again.`)
            return
        }
    }

    files = [...files, fi]
    dispatcher("fileAdded", fi)
}

function removeFile(file: FileInfo) {
    files = files.filter((f) => { return f !== file })
}


// TODO: revokeObjectURL()
// See: https://developer.mozilla.org/en-US/docs/Web/API/URL/createObjectURL


</script>

<script context="module">

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


</style>