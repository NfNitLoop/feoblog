<!--
    Shows a saved login, allows editing and switching IDs
    emits events:

    change - when values here are changed (and valid)
    checked - user checked this box
    unchecked - user unchecked this box.
    remove - remove button clicked
   
-->

<div class="item" class:collapsed transition:slide|local>
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
            <div>
                <Button disabled={first} on:click={() => dispatch("up")}>⬆️</Button>
                <Button disabled={last} on:click={() => dispatch("down")}>⬇️</Button>
            </div>
            <Button on:click={removeMe}>Remove</Button>
            <ColorPicker bind:color={savedLogin.bgColor} on:change={changed}/>
        </action-bar>

        <setting-section class={`securityLevel${securityLevelID}`}>
            <h3>Security Level: {securityLevel.name}</h3>
            <action-bar>
                <Button on:click={() => editingSecurity = !editingSecurity }>{#if editingSecurity}Cancel{:else}Edit{/if}</Button>
            </action-bar>

            {#if editingSecurity}
            <div>
                <p>Select your security level: 
                <input type="range" min="0" max="{securityLevels.length - 1}" bind:value={securityLevelID} >
            </div>

            <dual-pane>
                <pane-content>
                    <h1>Pros</h1>
                    <ul>
                    {#each securityLevel.pros as pro}
                        <li>{pro}</li>
                    {/each}
                    </ul>
                </pane-content>
                <pane-content>
                    <h1>Cons</h1>
                    <ul>
                    {#each securityLevel.cons as con}
                        <li>{con}</li>
                    {/each}
                    </ul>
                </pane-content>
                <pane-content>
                    <h1>Remember</h1>
                    <ul>
                        <li>The server never receives a copy of your private key or password.</li>
                        <li>You should never share your private key or password with anyone.</li>
                        <li>Regardless of security level, you MUST also store your private key in a separate, secure location. 
                            (Use a <a href="https://en.wikipedia.org/wiki/Password_manager">password manager</a>!)</li>
                    </ul>
                </pane-content>
            </dual-pane>

                {#if securityLevel.key == "insecure"}
                <div><SecretKeyInput
                    userID={UserID.fromString(savedLogin.userID)}
                    bind:value={privateKeyString}
                    bind:valid={validPrivateKey} 
                /></div>
                <Button disabled={!validPrivateKey} on:click={() => {
                    securityManager.setInsecure(savedLogin.userID.toString(), privateKeyString)
                    privateKeyString = ""
                    editingSecurity = false
                }}>Confirm</Button>
                {/if}
            
            {/if}

            <!-- <input type="password" value="*****************"/>
            <br><label>Save Password? <input type="checkbox"></label> -->

        </setting-section>

    </div>
</div>


<script lang="ts">

import { slide } from "svelte/transition"
import type { Writable } from "svelte/store"

import type { AppState, SavedLogin } from "../ts/app";
import { createEventDispatcher, getContext } from "svelte";
import Button from "./Button.svelte"
import ProfileImage from "./ProfileImage.svelte";
import { UserID } from "../ts/client";
import OpenArrow from "./OpenArrow.svelte";
import ColorPicker from "./ColorPicker.svelte";
import { securityLevels, SecurityManager } from "../ts/storage"
import SecretKeyInput from "./SecretKeyInput.svelte";

export let savedLogin: SavedLogin
export let checked = false
export let first = false
export let last = false

export let isOpen = checked

let appState: Writable<AppState> = getContext("appStateStore")
let securityManager = new SecurityManager(appState)

let checkbox: HTMLInputElement

let dispatch = createEventDispatcher()

let editingSecurity = false

let privateKeyString = ""
let validPrivateKey = false

$: securityLevelKey = securityManager.currentLevel(savedLogin)
$: securityLevelID = securityLevels.findIndex((l) => l.key == securityLevelKey)
$: securityLevel = securityLevels[securityLevelID] || securityLevels[securityLevels.length - 1]

$: collapsed = !isOpen

function headerClicked(event: MouseEvent) {
    // TODO: Make the whole header change logged-in user?
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

dual-pane {
    display: flex; 
    flex-wrap: wrap;
    gap: 1em;
}

dual-pane h1 , dual-pane ul{
    font-size: 1em;
    margin: 0px;
}

dual-pane ul {
    padding-left: 1em;
}

dual-pane pane-content {
    flex: 1;
}

dual-pane > * {
    min-width: 10em;
}

li:not(:first-of-type) {
    margin-top: 0.3em;
}

setting-section {
    display: block;
    background-color: #eee;
    padding: 1em;
    margin-top: 1em;
}
setting-section > *:first-child {
    margin-top: 0;
}

input[type="range"] {
    border: 0px;
}

action-bar {
    display: flex;
    width: 100%;
    justify-content: space-between;
    
    gap: 0.5rem;
}
</style>