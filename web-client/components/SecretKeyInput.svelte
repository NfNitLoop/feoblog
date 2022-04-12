<InputBox
    {label} {placeholder} {disabled}
    validationCallback={checkValue}
    inputType="password"
    bind:errorMessage
    bind:value    
    bind:hasFocus
/>

<script lang="ts">
import { PrivateKey, UserID } from "../ts/client";
import InputBox from "./InputBox.svelte";


export let placeholder = "Private Key"
export let value = ""
export let disabled = false
export let label = "Private Key"
export let valid = false
export let hasFocus = false

// If set, key must be correct for this user:
export let userID: UserID|null = null

let errorMessage = ""
$: valid = errorMessage === "" && value !== ""


// Error to display about the private key:
function checkValue(input: string) {
    if (input.length == 0) {
        return "";
    }
    
    // Throws string errors:
    let privateKey = PrivateKey.fromBase58(input)

    if (userID && userID.toString() != privateKey.userID.toString()) {
        return "Private key does not match user ID."
    }

    return ""    
}



</script>

