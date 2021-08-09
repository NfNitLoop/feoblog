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
    <div class="superTask entry" class:error class:warning class:isRunning on:click={clicked}>{$store.name}</div>

    {#if expanded}
    <div class="logs">
    {#each $store.logs as entry (entry)}
        {#if entry.subtask}
            <svelte:self tracker={entry.subtask} expanded={false}/>
        {:else}
            <div class="entry" class:error={entry.isError} class:warning={entry.isWarning}>{entry.message}</div>
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

.isRunning::after {
    content: " ...";
    animation: pulse 500ms linear 0s infinite alternate;
}

.error::before {
    content: "❌ ";
    line-height: 100%;
}

.warning::before {
    content: "⚠ ";
    line-height: 100%;
    font-weight: bold;
    color: orange;
}

.superTask:not(.error, .warning, .isRunning)::before {
    content: "✔ ";
    line-height: 100%;
    font-weight: bold;
    color: green;
}

.superTask:hover {
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