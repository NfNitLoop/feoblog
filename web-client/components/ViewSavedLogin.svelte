<script lang="ts">
/*
 * Shows a saved login, allows editing and switching IDs
 *
 * emits events:
 * change({savedLogin: SavedLogin}) when values here are changed (and valid)
 * logIn({savedLogin: SavedLogin}) when the user clicks the "Log In" button.
 * logOut({savedLogin: SavedLogin}) when the user clicks the "Log Out" button.
 */

import type { SavedLogin } from "../ts/app";
import { createEventDispatcher } from "svelte";
import Button from "./Button.svelte"

export let savedLogin: SavedLogin
export let isLoggedIn = false

// derived from savedLogin, so that we never modify it directly:
export let displayName = savedLogin.displayName || ""
export let userID = savedLogin.userID
export let bgColor = savedLogin.bgColor || ""

$: itemStyle = function(){
    if (isLoggedIn) {
        return ""
    }
    let color = bgColor
    if (!validColor(color)) {
        color = "rgba(0,0,0,0)"
    }
    return `border: 5px solid ${color};`
}()

function validColor(color: string): boolean {
    return (
        /^#[0-9a-f]{3}$/i.test(color) 
        || /^#[0-9a-f]{6}$/i.test(color) 
    )   
}

let dispatch = createEventDispatcher()

function logIn() {
    dispatch("logIn", eventData())
}

function remove() {
    dispatch("remove", eventData())
}

function onChange(...ignored: any) {
    if (bgColor && !validColor(bgColor)) return
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


<div class="savedLogin" >
<div class="item">
<div class="body" style={itemStyle}>
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
            <td><input class="color" type="text" bind:value={bgColor} placeholder="(none)"></td>
        </tr>
        <tr>
            <td></td>
            <td>
                {#if !isLoggedIn}<Button on:click={logIn}>Log In</Button>{/if}
                <Button on:click={remove} requiresConfirmation>Remove</Button>
            </td>
        </tr>
    </table>
</div>
</div>
</div>

<style>
input {
    border: 1px solid rgba(0, 0, 0, 0);
    font-family: inherit;
    font-size: inherit;
}
input:hover, input:focus {
    border: 1px solid black;
}

.userID, .color {
    font-family: Consolas, monospace;
}

</style>