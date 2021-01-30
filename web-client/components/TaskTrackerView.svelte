<!--
    Expand the logs from a TaskTracker into HTML.
    TODO:
      * Add collapsible sections.
      * Nicer styles
      * Recursive collapsible sections.
      * Allow parallel processing & logging in subsections.
-->
{#if $store.logs.length > 0}
<div class="taskTracker">
    <div class="taskName" class:error class:warning class:isRunning on:click={clicked}>{$store.name}</div>

    {#if expanded}
    <div class="logs">
    {#each $store.logs as entry (entry)}
        {#if entry.subtask}
            <svelte:self tracker={entry.subtask} expanded={false}/>
        {:else if entry.isError}
            <div class="entry error">{entry.message}</div>
        {:else if entry.isWarning}
            <div class="entry warning">⚠ {entry.message}</div>
        {:else}
            <div class="entry">{entry.message}</div>
        {/if}
    {/each}
    </div>
    {/if}

</div>
{/if}

<script lang="ts">
import { TaskTracker } from "../ts/common"
import { writable } from "svelte/store"

export let tracker = new TaskTracker()
export let expanded = true

// Put the tracker into a store so we can subscribe to update events:
let store = writable(tracker)
tracker.store = store
// Make changes to the tracker visibile to binds on TaskTrackerView:
$: tracker = $store

$: error = $store.errorCount > 0
$: warning = $store.warnCount > 0
$: isRunning = $store.isRunning

function clicked() {
    expanded = !expanded
}

</script>

<style>
div.taskTracker {
    font-size: 0.8rem;
    font-family: Consolas, monospace;
}

.taskName.isRunning::after {
    content: " ...";
    animation: pulse 500ms linear 0s infinite alternate;
}

.taskName.error::after {
    content: " ❌";
    line-height: 100%;
}

.taskName.warning::after {
    content: " ⚠";
    line-height: 100%;
}

.taskName:not(.error, .warning, .isRunning)::after {
    content: " ✔";
    line-height: 100%;
}

.taskName:hover {
    cursor: pointer;
    background-color: #eee;
}

.logs {
    padding-left: 1em;
    border-left: 1px solid #888;
    margin-bottom: 0.5em;
}


@keyframes pulse {
    to {
        opacity: 20%;
    }
}
</style>