<!-- 
    Page to edit the logged-in user's profile.
    Loads their existing profile first.
-->

{#await loadedProfile}
    <div class="item">Loading...</div>
{:then loaded} 
    {#if loaded.error}
        <div class="item error">{loaded.error}</div>
    {:else if !loaded.profile && !createNewOK}
        <div class="item">
            <p>Could not find an existing profile.</p>
            <Button on:click={createNew}>Create New Profile</Button>
        </div>
    {:else if !loaded.profile}
        <EditorWithPreview
            {appState}
            mode="profile"
        />
    {:else}
        <EditorWithPreview 
            {appState}
            mode="profile"
            initialItem={loaded.profile.item}
            signature={loaded.profile.signature.toString()}
        />
    {/if}
{:catch e} 
    <div class="item error">
        Error loading Profile. (See console)
    </div>
{/await}


<script lang="ts">
import type { Writable } from "svelte/store";

import type { AppState } from "../../ts/app";
import type { ProfileResult, UserID } from "../../ts/client";
import Button from "../Button.svelte";
import EditorWithPreview from "../EditorWithPreview.svelte"

export let appState: Writable<AppState>

$: userID = $appState.loggedInUser

let loadedProfile: Promise<LoadedProfile>
$: loadedProfile = loadProfile(userID)

let createNewOK = false

function createNew() {
    createNewOK = true
}

type LoadedProfile = {
    profile?: ProfileResult
    error?: string
}

async function loadProfile(userID: UserID|null): Promise<LoadedProfile> {
    console.log("userID", userID)
    console.log("$appState.loggedInUser", $appState.loggedInUser)
    if (!userID) return {
        error: "Must be logged in."
    }

    // TODO: Exhaustive search for latest profile.
    // Warn if we only got a profile from a non-exhaustive search.
    let result = await $appState.client.getLatestProfile(userID)

    if (result) return {
        profile: result
    }

    return {}
}


</script>