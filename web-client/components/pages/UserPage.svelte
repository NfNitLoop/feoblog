<!--
    Shows posts by a single user.
    TODO: use <ItemScroll>
-->
<PageHeading />

{#if !userID}
    <h1>Error: UserID is required</h1>
{:else}
    <ItemsScroll
        {createItemLoader}
        {itemFilter}
    />
{/if}

<script lang="ts">
import type { Writable } from "svelte/store";
import type { AppState } from "../../ts/app";
import { DisplayItem, ExcludeItemTypes, ItemOffsetParams } from "../../ts/client"

import { getContext } from "svelte";
import { params } from "svelte-hash-router"

import { UserID } from "../../ts/client";
import PageHeading from "../PageHeading.svelte";
import ItemsScroll from "../ItemsScroll.svelte";
import { ItemListEntry, ItemType } from "../../protos/feoblog";
import { ConsoleLogger } from "../../ts/common";

let appState: Writable<AppState> = getContext("appStateStore")

let items: DisplayItem[] = []
let endIsVisible: boolean

let loadingItems = true
const logger = new ConsoleLogger({prefix: "<UserPage>"})

$: userID = UserID.tryFromString($params.userID)


async function * createItemLoader(offset: ItemOffsetParams): AsyncGenerator<ItemListEntry> {
    if (!userID) { 
        logger.warn("Couldn't parse user ID, no items to load.")
        return
    }
    yield* $appState.client.getUserItems(userID, offset)
}

let itemFilter = new ExcludeItemTypes([ItemType.PROFILE])

</script>