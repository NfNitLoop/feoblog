<div class="input">
    <!-- svelte-ignore a11y-label-has-associated-control -->
    <label>
        {#if label}{label}: {/if}{#if errorMessage}<span class="error">{errorMessage}</span>{/if}
        <div>
        {#if inputType == "text" || inputType == "userID"}
            <input type="text" class:mono={inputType=="userID"} bind:value={value} {disabled} {placeholder} on:focus={gotFocus} on:blur={lostFocus} />
        {:else}
            <input 
                type="password" 
                autocomplete="current-password" 
                on:focus={gotFocus} on:blur={lostFocus}
                bind:value={value} 
                {disabled} 
                {placeholder} 
            />
        {/if}
        </div>
    </label>
</div>

<script lang="ts">
export let value = ""
export let disabled = false
export let label = ""
export let inputType: "text"|"userID"|"password" = "text"
export let placeholder = label
export let hasFocus = false

// Callback to validate the contents of the InputBox.
// Should return a non-empty error string if there's an error.
export let validationCallback: ((value: string) => string)|null = null

export let errorMessage = ""

$: checkValue(value)
function checkValue(value: string) {
    if (!validationCallback) { return }
    try {
        errorMessage = validationCallback(value)
    } catch (e) {
        if (typeof e === "string") {
            errorMessage = e
        } else {
            errorMessage = "Error in handler InputBox.validationCallback"
            console.error(errorMessage, e)
        }
    }
}

function gotFocus() { hasFocus = true }
function lostFocus() { hasFocus = false }

</script>

<style>
.mono {
    font-family: monospace;
}
</style>