<!-- 
    Page to edit the logged-in user's profile.
    Loads their existing profile first.
-->

{#await loadedProfile}
    <div class="item"><div class="body">Loading...</div></div>
{:then loaded} 
    {#if loaded.error}
        <div class="item"><div class="body error">{loaded.error}</div></div>
    {:else if !userID}
        <div class="error">userID is required</div>
    {:else}
        {#if !loaded.profile}
            <div class="item"><div class="body error">This user has no profile</div></div>
        {:else}
        <ItemView 
            item={loaded.profile.item}
            userID={userID.toString()}
            signature={loaded.profile.signature.toString()}
        />
        {/if}

        <div class="item">
            <div class="body">
                {#if userID.toString() == $appState.loggedInUser?.toString()} 
                        <Button href={`#/my_profile`}>Edit</Button>
                {/if}
                <Button href={`#/u/${userID}/`}>View Posts</Button>
            </div>
        </div>

    {/if}
{:catch e} 
    <div class="item"><div class="body error">
        Error loading Profile. (See console)
    </div></div>
{/await}


<script lang="ts">
import type { Writable } from "svelte/store";
import type { AppState } from "../../ts/app";

import { getContext } from "svelte";
import { params } from "svelte-hash-router"

import { ProfileResult, UserID } from "../../ts/client";
import Button from "../Button.svelte";
import ItemView from "../ItemView.svelte";

let appState: Writable<AppState> = getContext("appStateStore")

$: userID = UserID.tryFromString($params.userID)


let loadedProfile: Promise<LoadedProfile>
$: loadedProfile = loadProfile(userID)

type LoadedProfile = {
    profile?: ProfileResult
    error?: string
}

async function loadProfile(userID: UserID|null): Promise<LoadedProfile> {
    if (!userID) return {
        error: "UserID is required"
    }

    // Note: non-exhaustive search
    let result = await $appState.client.getProfile(userID)

    if (result) {
        return {
            profile: result
        }
    }

    return {}
}


</script>