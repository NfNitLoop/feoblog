<PageHeading />
{#if $appState.savedLogins}
    <ItemBox>
        <h1>Log In As:</h1>
        <saved-logins>
        {#each $appState.savedLogins as savedLogin, index (savedLogin.userID)}
        {@const uid = savedLogin.userID}
        {@const loggedIn = uid == $appState.loggedInUser?.asBase58}
        {@const isFirst = index == 0}
        {@const isLast = index == $appState.savedLogins.length - 1}
            <input type="checkbox" checked={loggedIn} on:change={(event) => checkClicked(event, savedLogin)}> <!--bind:this={checkbox} bind:checked on:click={checkClicked}> --> 
            <login-info >
                <ProfileImage userID={UserID.fromString(savedLogin.userID)}/>
                <input type="text" 
                        bind:value={savedLogin.displayName} 
                        placeholder="(unknown display name)"
                    >
                <user-id>id: {savedLogin.userID}</user-id>
                <ColorPicker bind:color={savedLogin.bgColor} on:change={() => update(savedLogin)}/>
                <Button disabled={isFirst} class="moveUp" on:click={() => move(savedLogin, "up")}>⬆️</Button>
                <Button disabled={isLast} class="moveDown" on:click={() =>  move(savedLogin, "down")}>⬇️</Button>
                <Button class="deleteLogin">❌</Button>
            </login-info>
        {/each}
        </saved-logins>
</ItemBox>
{:else}
<ItemBox>
    <p>FeoBlog allows you to save multiple identities which you can easily switch between. Why have one blog when you
        can have as many as you want! <tt>:)</tt>
    </p>
</ItemBox>        
{/if}

<hr/>

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
import ProfileImage from "../ProfileImage.svelte";
import ColorPicker from "../ColorPicker.svelte";


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

function checkClicked(e: Event, login: SavedLogin) {
    let newValue = (e.target as HTMLInputElement).checked
    if (newValue) {
        checked(login)
    } else {
        unchecked(login)
    }
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

<style>
saved-logins {
    display: grid;
    grid-template-columns: 0fr 10fr;
}

saved-logins > input {
    grid-column: 1;
    align-self: center;
}

login-info {
    display: grid;
    grid-template-columns: fit-content(5rem) fit-content(5rem) 1fr fit-content(5rem) fit-content(5rem);
    grid-auto-flow: dense;
    gap: 0.2rem;
    padding: 0.2em;
}

login-info:hover {
    background: #eee;
}

login-info > :global(.profileImage) {
    grid-row: 1 / span 2;
    align-self: center;
}

login-info :global(.colorPicker) {
    grid-column: 2;
    grid-row: 1 / span 2;
    align-self: center;
}

login-info > user-id {
    grid-row: 2;
}

login-info input {
    background: inherit;
}



login-info :global(.moveUp) {
    grid-row: 1 / span 2;
    grid-column: 4;
    align-self: center;
}

login-info :global(.moveDown) {
    grid-column: 5;
    grid-row: 1 / span 2;
    align-self: center;
}
login-info :global(.deleteLogin) {
    grid-column: 6;
    grid-row: 1 / span 2;
    align-self: center;
}



user-id {
    font-family: monospace;
    white-space: nowrap;
    overflow-x: hidden;
    text-overflow: ellipsis;
}


</style>