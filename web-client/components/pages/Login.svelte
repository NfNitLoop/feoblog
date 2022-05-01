<PageHeading />

{#each $appState.savedLogins as savedLogin, index (savedLogin.userID)}
    <div animate:flip={{duration: 200}}>
    <ViewSavedLogin 
        {savedLogin}
        checked={savedLogin.userID == $appState.loggedInUser?.toString()}
        on:change={() => update(savedLogin)}
        on:checked={() => checked(savedLogin)}
        on:unchecked={() => unchecked(savedLogin)}
        on:remove={() => removeID(savedLogin)}
        on:up={() => move(savedLogin, "up")}
        on:down={() => move(savedLogin, "down")}
        first={index == 0}
        last={index == $appState.savedLogins.length - 1}
    />
    </div>
{:else}
<ItemBox>
    <p>
    FeoBlog allows you to save multiple identities which you can easily switch between. Why have one blog when you can have as many as you want! :)
    </p>
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
import { flip } from "svelte/animate"

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

        if (!state.loggedIn) {
            state.logIn(login)
        }

        return state
    })

    reset()
}

function reset() {
    profileLoad = null
    userID = ""
}

function update(login: SavedLogin) {
    updateApp((app) => {
        app.updateSavedLogin(login)
    })
}

function removeID(savedLogin: SavedLogin) {
    updateApp((state) => {
        state.forgetLogin(savedLogin.userID)
    })
}

function checked(login: SavedLogin) {
    updateApp((app) => {
        app.logIn(login)
    })
}

function unchecked(login: SavedLogin) {
    updateApp((app) => {
        app.logOut()
    })
}

function move(login: SavedLogin, direction: "up"|"down") {
    let logins = [...$appState.savedLogins]
    let pos = logins.findIndex((l) => l.userID.toString() == login.userID.toString())

    if (direction == "up") {
        if (pos == 0) {
            console.warn("Can't move login up, already at top:", login)
            return
        }
        swap(logins, pos, pos - 1) 
    } else { // direction == "down"
        if (pos == logins.length - 1) {
            console.warn("Can't move login down, already at end:", login)
            return
        }
        swap(logins, pos, pos + 1)
    }

    updateApp((app) => {
        app.updateSavedLogins(logins)
    })
}

function updateApp(callback: (app: AppState) => void) {
    appState.update((app) => {
        callback(app)
        return app
    })
}

function swap<T>(arr: T[], index1: number, index2: number) {
    if (index1 == index2) { return }
    let tmp = arr[index1]
    arr[index1] = arr[index2]
    arr[index2] = tmp
}



</script>