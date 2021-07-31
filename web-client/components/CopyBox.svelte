<!--
    A box that allows you to easily copy something to the clipboard.
-->
<copy-box>
    <input 
        bind:this={inputBox}
        on:blur={inputBlur}
        on:click={inputClicked}
        type="text"
        value={value}
    ><copy-label 
        on:click={buttonClick} 
        class:recentlyClicked
    >{label}</copy-label>
</copy-box>

<script lang="ts">
export let value: string
export let label = "Copy"


let recentlyClicked = false

let existingTimer: number|null = null
let inputBox: HTMLInputElement

async function buttonClick() {
    const cb = navigator.clipboard
    if (!cb) {
        console.error("navigator.clipboard not found")
        return
    }

    try {
        await cb.writeText(value)
    } catch (error) {
        console.error("Couldn't copy to clipboard", error)
        return
    }

    recentlyClicked = true
    
    if (existingTimer) { clearTimeout(existingTimer) }
    existingTimer = setTimeout(
        () => { recentlyClicked = false },
        300
    )
}

function inputClicked() {
    console.log("input clicked")
    inputBox.selectionStart = 0
    inputBox.selectionEnd = inputBox.value.length
}

function inputBlur() {
    inputBox.value = value
}


</script>


<style>
copy-box {
    display: flex;
    align-items: stretch;
    font-weight: bold;
    --box-radius: 5px;
}

input, copy-label {
    background: white;
    border: 1px solid #888;
    padding: 0.5rem;
}


input {
    flex-shrink: 1;
    flex-grow: 1;
    font-size: 1rem;
    font-family: Consolas, monospace;
    border-top-left-radius: var(--box-radius);
    border-bottom-left-radius: var(--box-radius);
}

copy-label {
    border: 1px solid #888;
    border-left: 0px;
    border-top-right-radius: var(--box-radius);
    border-bottom-right-radius: var(--box-radius);
    cursor: pointer;
    box-shadow: inset 0px 0px 2px 1px rgb(0 0 0 / 20%);
    user-select: none;
}

copy-label:active {
    box-shadow: inset 0px 0px 5px 1px rgb(0 0 0 / 20%);
}

copy-label.recentlyClicked {
    /* animation: blinkGreen 100ms linear; */
    background: green;
    color: white;
}

@keyframes blinkGreen {
    50% {}
    51% {
        background: green;
        color: white;
    }
}


</style>