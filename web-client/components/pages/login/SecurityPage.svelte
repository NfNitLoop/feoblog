<PageHeading />

<ItemBox>
{#if !editingSecurity}
        <p><Button on:click={() => editingSecurity = true }>Edit</Button> Security Level: {currentSecurity.score}%</p>
{:else}
<setting-section>
    <h3>
        New Security Level: {securityRating.score}%
    </h3>
        
    <label><input type="checkbox" bind:checked={savePrivateKey}>Save my private key</label>
    {#if savePrivateKey}
    <div>
        <SecretKeyInput
            userID={UserID.fromString(savedLogin.userID)}
            bind:value={privateKeyString}
            bind:valid={validPrivateKey} 
            label=""
        />
        <label><input type="checkbox" bind:checked={saveWithPassword}>With a password</label>
        {#if saveWithPassword}
            <InputBox placeholder="Password" inputType="password" bind:value={keyPassword}/>
        {/if}
    </div>
    {/if}

    {#if tempEnabled}
        <label><input type="checkbox" bind:checked={saveTemporarily}>Temporarily remember my key after use</label>
        {#if saveTemporarily}
            <p>For up to {saveTimeSpan}</p>
            <input type="range" min=0 max={timeSpans.length - 1} bind:value={saveTimeSpanIndex}/>
        {/if}
    {/if}

    {#if errors.length > 0}
        <h3>Errors:</h3>
        <ul>
            {#each errors as error}
                <li>{error}</li>
            {/each}
        </ul>
    {:else}
        <security-pane>
            <security-section>
                <h1>Pros:</h1>
                <ul>
                {#each securityRating.pros as detail}
                    <li>{detail}</li>
                {/each}
                </ul>
            </security-section>
            <security-section>
                <h1>Cons:</h1>
                <ul>
                    {#each securityRating.cons as detail}
                        <li>{detail}</li>
                    {/each}
                    </ul>
            </security-section>
            <security-section>
                <h1>Remember:</h1>
                <ul>
                    {#each securityRating.remember as detail}
                        <li>{detail}</li>
                    {/each}
                    </ul>
            </security-section>
        </security-pane>
    {/if}

    <action-bar>
        <Button disabled={errors.length > 0} on:click={confirmSecuritySettings}>Confirm</Button>            
        <Button on:click={() => editingSecurity = false}>Cancel</Button>
    </action-bar>
</setting-section>
{/if}
</ItemBox>

<script lang="ts">
import type { Writable } from "svelte/store"
import type { AppState, SavedLogin } from "../../../ts/app";
import { createEventDispatcher, getContext } from "svelte";
import { UserID } from "../../../ts/client";
import { SecurityManager, SecurityManagerOptions, SecurityRating } from "../../../ts/storage"

import Button from "../../Button.svelte"
import SecretKeyInput from "../../SecretKeyInput.svelte";
import InputBox from "../../InputBox.svelte";
import ItemBox from "../../ItemBox.svelte";
import PageHeading from "../../PageHeading.svelte";

export let checked = false

export let isOpen = checked

let appState: Writable<AppState> = getContext("appStateStore")

$: userId = $appState.requireLoggedInUser()
$: savedLogin = function() {
    let login = $appState.getSavedLogin(userId.asBase58)
    if (!login) throw new Error("No saved login for this ID?")
    return login
}()

let editingSecurity = false

let privateKeyString = ""
let validPrivateKey = false

let savePrivateKey = false
let saveWithPassword = true
let keyPassword = ""

// Every time we uncheck savePrivateKey or saveWithPassword, unset the corresponding (potentially sensitive!) data:
$: if (!editingSecurity || !savePrivateKey) privateKeyString = ""
$: if (!editingSecurity || !savePrivateKey || !saveWithPassword) keyPassword = ""

let saveTemporarily = false
// Human readable timespan:
let saveTimeSpan = ""
let saveTimeSpanIndex = 0


const timeSpans = [
    // {secs: 60, text: "1 minute"},
    {secs: 60 * 5, text: "5 minutes"},
    {secs: 60 * 15, text: "15 minutes"},
    {secs: 60 * 60, text: "1 hour"},
    {secs: 60 * 60 * 4, text: "4 hours"},
    {secs: 60 * 60 * 24, text: "24 hours"},
    {secs: 60 * 60 * 24 * 7, text: "7 days"},
] as const

$: securityManager = new SecurityManager(appState, $appState)
$: currentSecurity = securityManager.getSettings(savedLogin.userID)

$: saveTimeSpan = timeSpans[saveTimeSpanIndex].text

$: collapsed = !isOpen



function confirmSecuritySettings() {
    securityManager.applySettings(securityOptions)
    reset()
}

function reset() {
    editingSecurity = false
    savePrivateKey = false
    saveWithPassword = true
    keyPassword = ""
    privateKeyString = ""
    saveTemporarily = false
    saveTimeSpanIndex = 0
}

let securityRating: SecurityRating
let errors: string[] = []

// No point in saving a temporary password if you're not encrypting:
$: tempEnabled = !(savePrivateKey && !saveWithPassword)
$: if (!tempEnabled) saveTemporarily = false

let securityOptions: SecurityManagerOptions
$: securityOptions = {
    userID: savedLogin.userID,
    privateKeyBase58Check: savePrivateKey ? privateKeyString : undefined,
    privateKeyPassword: saveWithPassword ? keyPassword : undefined,
    rememberKeySecs: saveTemporarily ? timeSpans[saveTimeSpanIndex].secs : undefined,
}
$:{
    let result = securityManager.calculateLevel(securityOptions)
    securityRating = result
    errors = result.errors   
}

</script>
    
    
<style>
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

setting-section label {
    display: block;
    /* padding-left: 2em; */
}

action-bar {
    display: flex;
    width: 100%;
    justify-content: space-between;
    gap: 0.5rem;
    flex-wrap: wrap;
}

security-pane {
    display: flex;
    flex-wrap: wrap;
    gap: 1em;
    margin-top: 1em;
}
security-pane > * {
    flex: 1;
    min-width: 15em;
}

security-pane h1 {
    margin-top: 0;
    font-size: 1em;
}

security-pane ul {
    padding-left: 1em;
    margin-top: 0;
}


</style>