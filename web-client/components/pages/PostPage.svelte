<!--
    Write a new post. 
-->

<EditorWithPreview {appState} mode="post"/>

<script lang="ts">
import type { Writable } from "svelte/store"
import type { AppState } from "../../ts/app";

import EditorWithPreview from "../EditorWithPreview.svelte"

export let appState: Writable<AppState>
</script>