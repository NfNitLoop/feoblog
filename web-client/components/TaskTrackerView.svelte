<!--
    Expand the logs from a TaskTracker into HTML.
    TODO:
      * Add collapsible sections.
      * Nicer styles
      * Recursive collapsible sections.
      * Allow parallel processing & logging in subsections.
-->
{#if $tracker.logs.length > 0}
<ul>
    {#each $tracker.logs as entry}
        {#if entry.isError}
            <li class=error>{entry.message}</li>
        {:else if entry.isWarning}
            <li>⚠ {entry.message}</li>
        {:else}
            <li>{entry.message}</li>
        {/if}
    {/each}
</ul>
{/if}

<script lang="ts">
import { TaskTracker } from "../ts/common"
import type { Writable } from "svelte/store"
import { writable } from "svelte/store"

export let tracker: Writable<TaskTracker> = writable(new TaskTracker())
</script>

<style>
ul {
    font-size: 0.8em;
    font-family: Consolas, monospace;
}
</style>