<!--
    If the user is "logged in", go to their feed page by default.
    Otherwise, go to the "home" page.
-->

<script lang="ts">
import { getContext, onMount } from "svelte";
import type { Writable } from "svelte/store";
import { push as navigateTo } from "svelte-spa-router"
import type { AppState } from "../../ts/app";

const appState: Writable<AppState> = getContext("appStateStore")

onMount(redirect)
async function redirect() {
    const userID = $appState.loggedInUser
    if (userID) {
        navigateTo(`#/u/${userID}/feed`)
        return
    }

    navigateTo(`#/home`)
}
</script>