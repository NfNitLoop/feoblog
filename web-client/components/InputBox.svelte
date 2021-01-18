<div class="input">
    <!-- svelte-ignore a11y-label-has-associated-control -->
    <label>
        {label}: {#if errorMessage}<span class="error">{errorMessage}</span>{/if}
        <div>
        {#if inputType == "text"}
            <input type="text" bind:value={value} {disabled} {placeholder} />
        {:else}
            <input type="password" bind:value={value} {disabled} {placeholder} />
        {/if}
        </div>
    </label>
</div>

<script lang="ts">
export let value = ""
export let disabled = false
export let label = ""
export let inputType: "text"|"password" = "text"
export let placeholder = label

// Callback to validate the type of the
export let validationCallback: undefined|((value: string) => string)

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

</script>