<div class="inputsGreyBox" in:fade out:fade>
    <InputBox
        label="User ID"
        validationCallback={parseUserIDError}
        bind:value={userID}
    />

    <InputBox
        label="Nickname"
        bind:value={displayName}
    />

    <div class="buttons">
        <Button on:click={() => dispatcher("delete")} requiresConfirmation>Unfollow</Button>
    </div>
</div>



<script lang="ts">

import { createEventDispatcher } from "svelte";
import { fade } from "svelte/transition"
import Button from "./Button.svelte"
import { parseUserIDError } from "../ts/common";
import InputBox from "./InputBox.svelte";

export let userID = ""
export let displayName = ""
let checked = false

let dispatcher = createEventDispatcher()

$: idError = parseUserIDError(userID)

</script>

