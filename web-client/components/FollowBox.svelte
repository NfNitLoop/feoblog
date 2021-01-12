<div class="followBox" in:fade out:fade>
    <table>
        <tr>
            <th></th>
            <td class="delete">
                {#if checked}
                    <Button on:click={() => dispatcher("delete")}>Unfollow</Button>
                {/if}
                <input type="checkbox" bind:checked>
            </td>
        </tr>
        <tr>
            <th>UserID:</th>
            <td>
                <input type="text" class="userID" bind:value={userID}>
                {#if idError}
                <div class="error">{idError}</div>
                {/if}
            </td>
        </tr>
        <tr>
            <th>Display&nbsp;Name:</th>
            <td><input type="text" bind:value={displayName}></td>
        </tr>
    </table>
</div>



<script lang="ts">

import { createEventDispatcher } from "svelte";
import { fade } from "svelte/transition"
import Button from "./Button.svelte"
import { parseUserIDError } from "../ts/common";

export let userID = ""
export let displayName = ""
let checked = false

let dispatcher = createEventDispatcher()

$: idError = parseUserIDError(userID)

</script>

<style>
    .followBox {
        border: 1px solid black;
        margin: 1px;
        margin-bottom: 1em;
    }
    th, td {
        vertical-align: top;
    }
    th {
        text-align: right;
        width: auto;
    }
    td {
        width: 100%;
    }
    td.delete {
        text-align: right;
    }

    table, input[type=text] {
        width: 100%;
    }

    .error { 
        color: red; 
        font-weight: bold;
    }

</style>