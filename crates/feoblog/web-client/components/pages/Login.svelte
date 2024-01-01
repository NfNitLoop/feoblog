<PageHeading />

{#if $appState.savedLogins}
    <ItemBox>
        <h1>Log In As:</h1>
        {#each $appState.savedLogins as savedLogin, index (savedLogin.userID)}
        {@const uid = savedLogin.userID}
        {@const loggedIn = uid == $appState.loggedInUser?.asBase58}
        {@const isFirst = index == 0}
        {@const isLast = index == $appState.savedLogins.length - 1}
        <saved-login animate:flip={{duration: 200}}>
            <input type="checkbox" checked={loggedIn} on:change={(event) => checkClicked(event, savedLogin)}> <!--bind:this={checkbox} bind:checked on:click={checkClicked}> --> 
                <ProfileImage userID={UserID.fromString(savedLogin.userID)}/>
                <input type="text" 
                        bind:value={savedLogin.displayName} 
                        placeholder="(unknown display name)"
                    >
                <user-id>id: {savedLogin.userID}</user-id>
                <ColorPicker bind:color={savedLogin.bgColor} on:change={() => update(savedLogin)}/>
                <Button disabled={isFirst} class="moveUp" on:click={() => move(savedLogin, "up")}>⬆️</Button>
                <Button disabled={isLast} class="moveDown" on:click={() =>  move(savedLogin, "down")}>⬇️</Button>
                <Button class="deleteLogin" on:click={() => removeID(savedLogin)}>❌</Button>
        </saved-login>
        {/each}
</ItemBox>
{:else}
<ItemBox>
    <p>FeoBlog allows you to save multiple identities which you can easily switch between. Why have one blog when you
        can have as many as you want! <tt>:)</tt>
    </p>
    <p>Use the "Add User ID" form below to add a user ID to this list. Then you can choose among these IDs to interact with
        content in FeoBlog.
    </p>
</ItemBox>        
{/if}

<ItemBox>
    <h1>Add User ID</h1>
    <form>
        <UserIDInput label="" placeholder="User ID" bind:value={userID} bind:valid={validUserID} bind:hasFocus />
    </form>

    {#if validUserID}
    <p>
        {#await profileLoad}
            Loading profile ...
        {:then profile}
                {#if profile} 
                    Add "{getInner(profile, "profile")?.displayName}"?
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

import type { AppState, SavedLogin } from "../../ts/app"
import { UserID, protobuf as pb, getInner } from "../../ts/client"
import UserIDInput from "../UserIDInput.svelte"
import Button from "../Button.svelte"
import PageHeading from "../PageHeading.svelte";
import ItemBox from "../ItemBox.svelte";
import ProfileImage from "../ProfileImage.svelte";
import ColorPicker from "../ColorPicker.svelte";


let appState: Writable<AppState> = getContext("appStateStore")
let userID = ""
let validUserID = false
let hasFocus = false

let profileLoad: null | Promise<pb.Item|null> = null

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
    let item = await profileLoad
    let displayName = getInner(item, "profile")?.displayName

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
saved-login {
    display: grid;
    grid-template-columns: repeat(3,fit-content(5rem)) 5fr repeat(3,fit-content(5rem));
    grid-template-areas:
        "check bg icon name up down del"
        "check bg icon id   up down del"
    ;
    align-items: center;
    gap: 0.2rem;
    padding: 0.2rem;
}

saved-login > input[type="checkbox"] {
    grid-area: check;
}

saved-login:hover {
    background: #eee;
}

saved-login > :global(.profileImage) {
    grid-area: icon;
}

saved-login :global(.colorPicker) {
    grid-area: bg;
}

saved-login > user-id {
    grid-area: id;
}

saved-login input[type="text"] {
    background: inherit;
    grid-area: name;
}

saved-login :global(.moveUp) {
    grid-area: up;
}

saved-login :global(.moveDown) {
    grid-area: down;
}
saved-login :global(.deleteLogin) {
    grid-area: del;
}

user-id {
    font-family: monospace;
    white-space: nowrap;
    overflow-x: hidden;
    text-overflow: ellipsis;
}


@media(max-width: 30rem) {
    saved-login {
        grid-template-columns: repeat(1,fit-content(5rem)) repeat(5,1fr);
        grid-template-areas:
            "check  name name name name name"
            "check  id   id   id   id   id"
            ".      icon bg   up   down del"
        ;
    }
}
</style>