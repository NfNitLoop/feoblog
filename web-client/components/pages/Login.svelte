<PageHeading breadcrumbs={getBreadcrumbs()} />

<!-- Browse As: -->
<div class="item">
    <div class="body">
    <form>
        <UserIDInput label="Browse as" bind:value={userID} bind:valid={validUserID} bind:hasFocus />
    </form>

    <p>
    {#if validUserID}
        {#await profileLoad}
            Loading profile ...
        {:then profile}
                {#if profile} 
                    Browse as user "{profile.profile?.display_name}"?
                {:else}
                    Could not load profile for that user ID. Log in anwyay?
                {/if}
        {:catch err}
            Error: {err}
        {/await}
    {/if}
    </p>
    <Button on:click={confirmLogin} disabled={!validUserID}>Confirm</Button>

    
    </div>
    </div>

<CreateID></CreateID>

{#each $appState.savedLogins as savedLogin, index (savedLogin.userID)}
    <ViewSavedLogin {savedLogin} isLoggedIn={index===0} on:logIn={logInSaved} on:remove={removeSaved} on:change={updateSavedLogin}/>
{:else}
<div class="item">
<div class="body">
        You are not currently logged in.
</div>
</div>
{/each}






<script lang="ts">
import { getContext } from "svelte";
import type { Writable } from "svelte/store"
import type { Item, Profile } from "../../protos/feoblog"
import type { AppState, SavedLogin } from "../../ts/app"
import { UserID } from "../../ts/client"
import UserIDInput from "../UserIDInput.svelte"
import Button from "../Button.svelte"
import ViewSavedLogin from "../ViewSavedLogin.svelte"
import CreateID from "./CreateID.svelte"
import PageHeading from "../PageHeading.svelte";

let appState: Writable<AppState> = getContext("appStateStore")
let userID = ""
let validUserID = false
let hasFocus = false

let profileLoad: null | Promise<Item|null> = null

$: if (userID && validUserID) { fetchProfile() }

// Get rid of the manual login here and just load profile when we have a possibly-valid userID.
function fetchProfile() {
    // Load the user profile.
    let client = $appState.client

    let loadProfile = async () => {
        let result = await client.getProfile(UserID.fromString(userID))
        return result?.item || null
    }
    profileLoad = loadProfile()
}

async function confirmLogin() {
    // Log in via app state.

    let login: SavedLogin = {userID}
    let displayName = (await profileLoad)?.profile?.display_name

    appState.update((state) => {
        if (displayName) { login.displayName = displayName}
        state.logIn(login)
        return state
    })

    reset()
}

function reset() {
    profileLoad = null
    userID = ""
}

type LoginChangeEvent = {
    detail: {
        savedLogin: SavedLogin
    }
}

function logInSaved(event: LoginChangeEvent) {
    let savedLogin: SavedLogin = event.detail.savedLogin
    appState.update((state) => {
        state.logIn(savedLogin)
        return state
    })
}

function removeSaved(event: LoginChangeEvent) {
    let savedLogin: SavedLogin = event.detail.savedLogin
    appState.update((state) => {
        state.forgetLogin(savedLogin.userID)
        return state
    })
}
function updateSavedLogin(event: LoginChangeEvent) {
    let savedLogin: SavedLogin = event.detail.savedLogin
    appState.update((state) => {
        state.updateSavedLogin(savedLogin)
        return state
    })
}

function getBreadcrumbs() {

    return {
        crumbs: [
            {text: "Log In"}
        ]
    }
}

</script>