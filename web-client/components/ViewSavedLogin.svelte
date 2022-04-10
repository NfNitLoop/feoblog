<!--
    Shows a saved login, allows editing and switching IDs
    emits events:

    change - when values here are changed (and valid)
    checked - user checked this box
    unchecked - user unchecked this box.
    remove - remove button clicked

    
-->

<div class="item" class:collapsed>
    <div class="header" on:click={headerClicked}>
        <input type="checkbox" bind:this={checkbox} bind:checked on:click={checkClicked}>
        <ProfileImage userID={UserID.fromString(savedLogin.userID)} slot="headerLeft"/>
        <div class="mid">
            <input type="text" 
                bind:value={savedLogin.displayName} 
                placeholder="(unknown display name)"
                on:change={changed}
            >
            <br><span class="userID">id: {savedLogin.userID}</span>    
        </div>
        <OpenArrow bind:isOpen/>
    </div>
    <div class="body">
        <action-bar>
            <Button href="#/u/{savedLogin.userID}/feed">View Feed</Button>
            <Button href="#/u/{savedLogin.userID}/">View Posts</Button>
            <Button href="#/u/{savedLogin.userID}/profile">View Profile</Button>
            <Button on:click={removeMe}>Remove</Button>
            <ColorPicker bind:color={savedLogin.bgColor} on:change={changed}/>
        </action-bar>

    </div>
</div>


<script lang="ts">

import type { SavedLogin } from "../ts/app";
import { createEventDispatcher } from "svelte";
import Button from "./Button.svelte"
import ProfileImage from "./ProfileImage.svelte";
import { UserID } from "../ts/client";
import OpenArrow from "./OpenArrow.svelte";
import ColorPicker from "./ColorPicker.svelte";

export let savedLogin: SavedLogin
export let checked = false

export let isOpen = false

let checkbox: HTMLInputElement

let dispatch = createEventDispatcher()

$: collapsed = !isOpen

function headerClicked(event: MouseEvent) {
    // TODO: Make the whole header change logged-in user.
}

function changed() {
    dispatch("change")
}

function checkClicked() {
    let newValue = !checked
    let action = newValue ? "checked" : "unchecked"
    if (newValue) {
        isOpen = true
    }
    dispatch(action)
}

function removeMe() {
    dispatch("remove")
}

</script>


<style>
.userID {
    font-family: monospace;
}

.header input {
    background: inherit;
    border: 0px;
}

.header .mid {
    flex-grow: 1;
}


.item .header {
    padding: 0.5rem 1.0rem;
}

.item.collapsed .body {
    display: none
}
.item.collapsed .header {
    border-radius: 20px;
}

action-bar {
    display: flex;
    width: 100%;
    justify-content: space-between;
    
    gap: 0.5rem;
}
</style>