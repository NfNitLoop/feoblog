<div class="item">
{#if !loggedIn}
    <p>You are not logged in.</p>
    <form>
        Log in as:
        <br><UserIDInput bind:value={userID} bind:valid={validUserID}/>
        <br><button on:click|preventDefault={logIn} disabled={!validUserID}>Log in</button>
    </form>

    {#if errorMessage != ""}
        <p>Error: {errorMessage}</p>
    {/if}

    {#if attemptedProfileLoad}
        {#if profile} 
            <p>Log in as user "{profile.profile?.display_name}"?</p>
        {:else}
            <p>Could not load profile for that user ID. Log in anwyay?</p>
        {/if}
        <button on:click|preventDefault={confirmLogin}>Confirm</button>
    {/if}
{:else}
    <p>You are logged in.
    <ul>
        <li>Name: {$appState.userName || "(unknown user)"}
        <li>userID: <span class="userID">{$appState.loggedInUser}</span>
    </ul>
    <button on:click|preventDefault={logout}>Log out</button>
{/if}
</div>

<script lang="ts">
import type { Writable } from "svelte/store";
import type { Item } from "../../protos/feoblog";
import type { AppState, getInstance } from "../../ts/app"
import { Client, UserID } from "../../ts/client";
import UserIDInput from "../UserIDInput.svelte"

export let appState: Writable<AppState>
let userID = ""
let validUserID = false
let errorMessage = ""
$: loggedIn = $appState.loggedIn
$: loginEnabled = validUserID // TODO && validProfile

let attemptedProfileLoad = false
let profile: Item|null = null

async function logIn() {
    // Load the user profile.
    let client = new Client({
        base_url: ""
    })

    try {
        attemptedProfileLoad = true
        profile = await client.getProfile(UserID.fromString(userID))
    } catch (exception) {
        console.log("error", exception)
        errorMessage = `${exception}`
    }
}
function confirmLogin() {
    // Log in via app state.
    appState.update((state) => {
        state.login(UserID.fromString(userID), profile)
        return state
    })

    reset()
}

function logout() {
    appState.update((state) => {
        state.logout()
        return state
    })

    reset()
}

function reset() {
    attemptedProfileLoad = false
    profile = null
    userID = ""
}
</script>