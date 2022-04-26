<!--
    If the user is "logged in", go to their feed page by default.
    Otherwise, go to the "home" page.
-->

<script lang="ts">
import { getContext, onMount } from "svelte";
import type { Writable } from "svelte/store";
import type { AppState } from "../../ts/app";

const appState: Writable<AppState> = getContext("appStateStore")

onMount(redirect)
async function redirect() {
    const userID = $appState.loggedInUser
    if (userID) {
       window.location.hash = `#/u/${userID}/feed`
        return
    }

    window.location.hash = `#/home`
}
</script>