<div>
    <input type="text" class="userID" class:error {placeholder} bind:value {disabled}/>
    {#if error}
        <p class="errorMessage">{errorMessage}</p>
    {/if}
</div>

<script lang="ts">
import { UserID } from "../ts/client";


export let placeholder = "user ID"
export let value = ""
export let disabled = false

export let valid = false

$: error = errorMessage !== ""
$: errorMessage = function() {
    if (value === "") {
        return ""
    }

    try {
        UserID.fromString(value)
    } catch (exception) {
        return `Error: ${exception}`
    }

    return ""
}()
$: valid = !error && (value !== "")
</script>

<style>
    input.userID {
        width: 30em;
    }
    .error {
        border: 2px solid darkred;
    }
    .errorMessage {
        color: darkred;
        font-weight: bold;
        margin: 0px;
    }
</style>