<div class="inputsGreyBox">
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
        <Button on:click={() => dispatcher("delete")}>Unfollow</Button>
    </div>
</div>



<script lang="ts">

import { createEventDispatcher } from "svelte";
import Button from "./Button.svelte"
import { parseUserIDError } from "../ts/common";
import InputBox from "./InputBox.svelte";

export let userID = ""
export let displayName = ""

let dispatcher = createEventDispatcher()

</script>

