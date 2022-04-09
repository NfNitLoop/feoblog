<PageHeading />

{#each $appState.savedLogins as savedLogin, index (savedLogin.userID)}
    <ViewSavedLogin 
        {savedLogin}
        checked={savedLogin.userID == $appState.loggedInUser?.toString()}
        on:change={() => update(savedLogin)}
        on:checked={() => checked(savedLogin)}
        on:unchecked={() => unchecked(savedLogin)}
        on:remove={() => removeID(savedLogin)}
    />
{:else}
<ItemBox>
    FeoBlog allows you to save multiple identities which you can easily switch between. Why have one blog when you can have as many as you want! :)
</ItemBox>        
{/each}


<ItemBox>
    <form>
        <UserIDInput label="" placeholder="Add User ID" bind:value={userID} bind:valid={validUserID} bind:hasFocus />
    </form>

    {#if validUserID}
    <p>
        {#await profileLoad}
            Loading profile ...
        {:then profile}
                {#if profile} 
                    Add "{profile.profile?.display_name}"?
                {:else}
                    Could not load profile for that user ID. Add it anyway?
                {/if}
        {:catch err}
            Error: {err}
        {/await} 
    </p>
    {/if}
    {#if userID?.length > 0}
        <Button on:click={addUserID} disabled={!validUserID}>Confirm</Button>
    {/if}
</ItemBox>


<script lang="ts">
import { getContext } from "svelte";
import type { Writable } from "svelte/store"
import type { Item, Profile } from "../../protos/feoblog"
import type { AppState, SavedLogin } from "../../ts/app"
import { UserID } from "../../ts/client"
import UserIDInput from "../UserIDInput.svelte"
import Button from "../Button.svelte"
import ViewSavedLogin from "../ViewSavedLogin.svelte"
import PageHeading from "../PageHeading.svelte";
import ItemBox from "../ItemBox.svelte";


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

async function addUserID() {
    // Log in via app state.

    let login: SavedLogin = {userID}
    let displayName = (await profileLoad)?.profile?.display_name

    appState.update((state) => {
        if (displayName) { login.displayName = displayName}
        state.updateSavedLogin(login)
        return state
    })

    reset()
}

function reset() {
    profileLoad = null
    userID = ""
}

function update(login: SavedLogin) {
    appState.update((app) => {
        app.updateSavedLogin(login)
        return app
    })
}

function removeID(savedLogin: SavedLogin) {
    appState.update((state) => {
        state.forgetLogin(savedLogin.userID)
        return state
    })
}

function checked(login: SavedLogin) {
    appState.update((app) => {
            app.logIn(login)
            return app
    })
}

function unchecked(login: SavedLogin) {
    appState.update((app) => {
            app.logOut()
            return app
    })
}





</script>