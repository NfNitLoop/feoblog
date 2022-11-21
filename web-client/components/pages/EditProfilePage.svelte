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
        <PageHeading/>
        <div class="item">
            <div class="body">
                <p>Could not find an existing profile.</p>
                <Button on:click={createNew}>Create New Profile</Button>
                <Button href="#/u/{userID}/sync">Sync from another server</Button>
            </div>
        </div>
    {:else if !loaded.profile}
        <EditorWithPreview
            mode="profile"
        />
    {:else}
        <EditorWithPreview 
            mode="profile"
            initialItem={loaded.profile.item}
        />
    {/if}
{:catch e} 
    <div class="item error">
        Error loading Profile. (See console)
    </div>
{/await}


<script lang="ts">
import { getContext } from "svelte";

import type { Writable } from "svelte/store";

import type { AppState } from "../../ts/app";
import type { ProfileResult, UserID } from "../../ts/client";
    import { ConsoleLogger } from "../../ts/common";
import Button from "../Button.svelte";
import EditorWithPreview from "../EditorWithPreview.svelte"
import PageHeading from "../PageHeading.svelte";

let logger = new ConsoleLogger({prefix: "<EditProfilePage>"}) //.withDebug()
logger.debug("loaded")
let appState: Writable<AppState> = getContext("appStateStore")
let userID = $appState.loggedInUser

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
    if (!userID) return {
        error: "Must be logged in."
    }

    let result = await $appState.client.getProfile(userID)

    if (result) {
        logger.debug("Loaded profile for user: ", userID.asBase58)
        return {
            profile: result
        }
    }

    return {}
}


</script>