<script lang="ts">
/*
 * Shows a saved login, allows editing and switching IDs
 *
 * emits events:
 * changed({savedLogin: SavedLogin}) when values here are changed (and valid)
 * logIn({savedLogin: SavedLogin}) when the user clicks the "Log In" button.
 * logOut({savedLogin: SavedLogin}) when the user clicks the "Log Out" button.
 */

import type { SavedLogin } from "../ts/app";
import { createEventDispatcher } from "svelte";

export let savedLogin: SavedLogin
export let isLoggedIn = false

// derived from savedLogin, so that we never modify it directly:
export let displayName = savedLogin.displayName || ""
export let userID = savedLogin.userID
export let bgColor = savedLogin.bgColor || ""

let dispatch = createEventDispatcher()

function logIn() {
    dispatch("logIn", eventData())
}

function remove() {
    dispatch("remove", eventData())
}

function onChange(...ignored) {
    dispatch("change", eventData())
}

function eventData(): EventData {
    return {
        savedLogin: {
            userID,
            displayName,
            bgColor,
        }
    }
}

$: { displayName; bgColor; onChange() }


class EventData {
    savedLogin: SavedLogin
}
</script>


<div class="savedLogin" style="background-color: {bgColor};">
<div class="item" >
    <table>
        {#if isLoggedIn}
        <tr>
            <th colspan=2>Logged in as:</th>
        </tr>
        {/if}
        <tr>
            <td>Name:</td>
            <td><input type="text" bind:value={displayName} placeholder="(unknown display name)"></td>
        </tr>
        <tr>
            <td>User ID:</td>
            <td><span class="userID">{userID}</span></td>
        </tr>
        <tr>
            <td>Color:</td>
            <td><input type="text" bind:value={bgColor} placeholder="(none)"></td>
        </tr>
        <tr>
            <td></td>
            <td>
                {#if !isLoggedIn}<button on:click|preventDefault={logIn}>Log In</button>{/if}
                <button on:click|preventDefault={remove}>Remove</button>
            </td>
        </tr>
    </table>
</div>
</div>

<style>
    input {
        border: 1px solid rgba(0, 0, 0, 0)
    }
    input:hover, input:focus {
        border: 1px solid black;
    }
</style>