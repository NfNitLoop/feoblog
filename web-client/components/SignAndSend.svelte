<!-- 
    Component that knows how to sign and send an item 
    Accepts an Item, presents a sign & send box for the logged-in user.
    dispatches:
     * "sendSuccess", {userID, signature}
-->
<div class="sendBox inputWhiteBox" transition:slide|local>
    {#if errors.length > 0}
        <div class="error">
            {#each errors as error}
                {error}<br>
            {/each}
        </div>
    
    {:else if !validSignature}
        <InputBox 
            inputType="password"
            label="Private Key"
            placeholder=""
            bind:value={privateKey}
            bind:errorMessage={privateKeyError}
        />
        <Button on:click={sign} disabled={disabled || !privateKey || !validPrivateKey}>Sign</Button>
    {:else}
        <InputBox
            label="Signature"
            value={signature}
            disabled
         />
        <div class="buttons">
            <Button on:click={submit}>Submit</Button>
        </div>
        {#if status}
            <div>{status}</div>
        {/if}
    {/if}
</div>


<script lang="ts">
import { push as navigateTo } from "svelte-spa-router"
import { slide } from "svelte/transition"
import type { Writable } from "svelte/store";
import type { Item } from "../protos/feoblog";
import type { AppState } from "../ts/app";
import { Signature } from "../ts/client";
import Button from "./Button.svelte";
import InputBox from "./InputBox.svelte"
import { createEventDispatcher } from "svelte";
import bs58 from "bs58";
import bs58check from "bs58check"
import nacl from "tweetnacl";

export let appState: Writable<AppState>
export let item: Item
export let disabled = false
// Errors sent to us from outside.
export let errors: string[] = []
export let navigateWhenDone = true

$: itemBytes = item.serialize()

$: userID = $appState.requireLoggedInUser()

let privateKey = ""
let signature = ""
let status = ""


let dispatcher = createEventDispatcher()

$: validSignature = function(): boolean {
    if (!userID || !signature || !itemBytes) {
        return false
    }

    let isValid = false
    try {
        let sig = Signature.fromString(signature)
        isValid = sig.isValidSync(userID, itemBytes)
    } catch (error) {
        console.error("Error validating signature:", error)
    }

    // Re-validating a signature on every keypress is *expensive*.
    // If we've started editing and this signature is no longer valid, delete it so
    // that we can short-circuit (above)
    if (!isValid) {
        signature = ""
    }

    return isValid
}()

// Error to display about the private key:
$: privateKeyError = function() {
    if (privateKey.length == 0) {
        return "";
    }
    
    let buf: Uint8Array;
    try {
        buf = bs58.decode(privateKey)
    } catch (error) {
        return "Not valid base58"
    }

    // Secret is 32 bytes, + 4 for checked base58.
    if (buf.length < 36) {
        return "Password is too short."
    }
    if (buf.length > 36) {
        return "Password is too long."
    }

    try {
        buf = bs58check.decode(privateKey)
    } catch (e) {
        return "Invalid Password"
    }

    
    let keypair = nacl.sign.keyPair.fromSeed(buf);
    
    let pubKey = bs58.encode(keypair.publicKey)
    if (pubKey != userID.toString()) {
        return "Private key does not match user ID."
    }

    return ""    
}()
$: validPrivateKey = !privateKeyError


// Create a signature, delete the password.
function sign() {
    if (privateKeyError) {
        console.error("Shouldn't be able to call sign w/ invalid private key.")
        return
    }

    if (!itemBytes) throw `No bytes to sign.`

    let buf = bs58check.decode(privateKey)
    let keypair = nacl.sign.keyPair.fromSeed(buf);
    let binSignature = nacl.sign.detached(itemBytes, keypair.secretKey)
    signature = bs58.encode(binSignature)

    // Delete the privateKey, we don't want to save it any longer than
    // necessary:
    privateKey = ""
}

async function submit() {
    if ( (errors.length > 0) || !validSignature) {
        console.error("Submit clicked when not valid");
        return
    }

    if (!itemBytes || itemBytes.length == 0) {
        let msg = "Refusing to send 0 bytes"
        console.error(msg)
        status = msg
        return
    }

    let sig = Signature.fromString(signature)

    status = "Making request"

    let result 
    try {
        result = await $appState.client.putItem(userID, sig, itemBytes)
    } catch (e) {
        console.error("PUT exception:", e)
        status = `PUT exception: ${e}`
        return 
    }
    
    status = `Success: ${result.status}: ${result.statusText}`

    dispatcher("sendSuccess", {userID, signature: sig})

    if (navigateWhenDone) {
        navigateTo(`#/u/${userID}/i/${signature}/`)
    }
}

</script>