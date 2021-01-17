<!-- 
    Page to edit the logged-in user's profile.
    Loads their existing profile first.
-->

{#await loadedProfile}
    <div class="item">Loading...</div>
{:then loaded} 
    {#if loaded.error}
        <div class="item error">{loaded.error}</div>
    {:else if !loaded.profile}
        <div class="item error">This state should be unreachable</div>
    {:else}
        <ItemView 
            {appState}
            item={loaded.profile.item}
            userID={userID.toString()}
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
import { ProfileResult, UserID } from "../../ts/client";
import ItemView from "../ItemView.svelte";

export let appState: Writable<AppState>
export let params: {
    userID: string
}
let userID = UserID.fromString(params.userID)

let loadedProfile: Promise<LoadedProfile>
$: loadedProfile = loadProfile(userID)

type LoadedProfile = {
    profile?: ProfileResult
    error?: string
}

async function loadProfile(userID: UserID|null): Promise<LoadedProfile> {
    if (!userID) return {
        error: "Must be logged in."
    }

    // Note: non-exhaustive search
    let result = await $appState.client.getProfile(userID)

    if (result) return {
        profile: result
    }

    return {}
}


</script>