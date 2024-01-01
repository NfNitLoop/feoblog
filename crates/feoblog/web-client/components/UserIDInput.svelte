<InputBox
    {label} {placeholder} {disabled}
    validationCallback={checkValue}
    bind:errorMessage
    bind:value    
    bind:hasFocus
/>

<script lang="ts">
import { UserID } from "../ts/client";
import InputBox from "./InputBox.svelte";


export let placeholder = "user ID"
export let value = ""
export let disabled = false
export let label = "UserID"
export let valid = false
export let hasFocus = false

let errorMessage = ""
$: valid = errorMessage === "" && value !== ""


function checkValue(value: string): string{
    if (value === "") {
        return ""
    }
    UserID.fromString(value) // throws a string
    return ""
}

</script>

