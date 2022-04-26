<!--
    Shows posts by a single user.
    TODO: Implement infinite scroll like on the feed page.
-->

{#if userID}
    <PageHeading />

    {#each items as entry, index (entry.signature)}
    <ItemView 
    userID={entry.userID.toString()}
    signature={entry.signature.toString()}
    item={entry.item}
    />
    {:else}
    <div class="item">
        <div class="body">
            <p>
            No posts found for user <UserIDView {userID}/>
            </p>
        </div>
    </div>
    {/each}

    <VisibilityCheck on:itemVisible={() => lazyLoader?.displayMoreItems?.()} bind:visible={endIsVisible}/>
{:else}
<h1>Error: UserID is required</h1>
{/if}

<script lang="ts">
import type { Writable } from "svelte/store";
import type { AppState } from "../../ts/app";
import type { DisplayItem } from "../../ts/client"

import { getContext } from "svelte";
import { params } from "svelte-hash-router"

import { UserID, LazyItemLoader } from "../../ts/client";
import ItemView from "../ItemView.svelte"
import VisibilityCheck from "../VisibilityCheck.svelte";
import UserIDView from "../UserIDView.svelte"
import PageHeading, { Breadcrumbs, NavItem } from "../PageHeading.svelte";
import ProfileImage from "../ProfileImage.svelte";

let appState: Writable<AppState> = getContext("appStateStore")

let items: DisplayItem[] = []
let endIsVisible: boolean

let loadingItems = true


$: userID = UserID.tryFromString($params.userID)


$: lazyLoader = createLazyLoader(userID)
function createLazyLoader(userID: UserID|null) {
    items = []
    if (lazyLoader) { lazyLoader.stop() }
    if (!userID) { return }

    return new LazyItemLoader({
        client: $appState.client,
        itemEntries: $appState.client.getUserItems(userID),
        continueLoading: () => endIsVisible,
        endReached: () => { loadingItems = false },
        displayItem: async (di) => {
            if (di.item.profile) {
                // Don't display profile updates here.
                return
            }
            items = [...items, di]
        },
    })
}

</script>