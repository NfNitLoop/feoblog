{#each $appState.savedLogins as savedLogin, index (savedLogin.userID)}
    <ViewSavedLogin {savedLogin} isLoggedIn={index===0} on:logIn={logInSaved} on:remove={removeSaved} on:change={updateSavedLogin}/>
{:else}
<div class="item">
    You are not currently logged in.
</div>
{/each}

<div class="item">
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
</div>

<CreateID></CreateID>



<script lang="ts">
import type { Writable } from "svelte/store"
import type { Item } from "../../protos/feoblog"
import type { AppState, SavedLogin } from "../../ts/app"
import { Client, UserID } from "../../ts/client"
import UserIDInput from "../UserIDInput.svelte"
import ViewSavedLogin from "../ViewSavedLogin.svelte"
import CreateID from "./CreateID.svelte"

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
        let result = await client.getProfile(UserID.fromString(userID))
        profile = result?.item || null
    } catch (exception) {
        console.log("error", exception)
        errorMessage = `${exception}`
    }
}
function confirmLogin() {
    // Log in via app state.
    appState.update((state) => {
        let login: SavedLogin = {userID}
        let displayName = profile?.profile?.display_name
        if (displayName) { login.displayName = displayName}

        state.logIn(login)
        return state
    })

    reset()
}

function reset() {
    attemptedProfileLoad = false
    profile = null
    userID = ""
}

function logInSaved(event) {
    let savedLogin: SavedLogin = event.detail.savedLogin
    appState.update((state) => {
        state.logIn(savedLogin)
        return state
    })
}

function removeSaved(event) {
    let savedLogin: SavedLogin = event.detail.savedLogin
    appState.update((state) => {
        state.forgetLogin(savedLogin.userID)
        return state
    })
}
function updateSavedLogin(event) {
    let savedLogin: SavedLogin = event.detail.savedLogin
    appState.update((state) => {
        state.updateSavedLogin(savedLogin)
        return state
    })
}

</script>