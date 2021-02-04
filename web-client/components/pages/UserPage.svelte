<!--
    Shows posts by a single user.
-->

<div class="item">
    <h1>Posts by:</h1>
    <div class="userInfo">
        <UserIDView {appState} {userID}/>
    </div>
    <Button href={`#/u/${userID}/profile`}>View Profile</Button>
</div>

{#each items as entry, index (entry.signature)}
    <ItemView 
        userID={entry.userID.toString()}
        signature={entry.signature.toString()}
        item={entry.item}
        {appState}
    />
{:else}
    <div class="item">No posts found for user <UserIDView {appState} {userID}/></div>
{/each}

<VisibilityCheck on:itemVisible={lazyLoader.displayMoreItems} bind:visible={endIsVisible}/>

<script lang="ts">
import type { Writable } from "svelte/store";

import type { AppState } from "../../ts/app";
import type { DisplayItem } from "../../ts/client"
import { UserID, LazyItemLoader } from "../../ts/client";

import ItemView from "../ItemView.svelte"
import Button from "../Button.svelte"
import VisibilityCheck from "../VisibilityCheck.svelte";
import UserIDView from "../UserIDView.svelte"

export let appState: Writable<AppState>

let items: DisplayItem[] = []
let endIsVisible: boolean

let loadingItems = true

export let params: {
    userID: string
}
$: userID = UserID.fromString(params.userID)


$: lazyLoader = createLazyLoader(userID)
function createLazyLoader(userID: UserID) {
    return new LazyItemLoader({
        client: $appState.client,
        itemEntries: $appState.client.getUserItems(userID),
        endIsVisible: () => endIsVisible,
        endReached: () => { loadingItems = false },
        displayItem: (di) => {
            if (di.item.profile) {
                // Don't display profile updates here.
                return
            }
            items = [...items, di]
        },
    })
}

</script>