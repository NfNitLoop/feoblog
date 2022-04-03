<!--
    Shows posts by a single user.
    TODO: Implement infinite scroll like on the feed page.
-->

{#if userID}
    <PageHeading navItems={getNav(userID)} breadcrumbs={breadcrumbs(userID)} />

    {#each items as entry, index (entry.signature)}
    <ItemView 
    userID={entry.userID.toString()}
    signature={entry.signature.toString()}
    item={entry.item}
    />
    {:else}
    <div class="item">
        <div class="body">
            No posts found for user <UserIDView {userID}/>
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

function getNav(userID: UserID): NavItem[] {
    let app = $appState
    let nav = app.navigator
    let loggedIn = app.loggedInUser?.toString() == userID.toString()
    let my = loggedIn ? "My " : ""

    let items = [
        { text: `${my}Feed`, href: nav.userFeed(userID).hash },
        { text: `${my}Profile`, href: nav.userProfile(userID).hash },
    ]
    if (loggedIn) {
        items = items.concat([
            { text: "New Post", href: nav.newPost().hash },
            { text: "Sync", href: nav.sync().hash },
        ])
    }
    return items
}

function breadcrumbs(userID: UserID): Breadcrumbs {
    return {
        crumbs: [ { userID }, {text: "Posts"} ]
    }
}


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