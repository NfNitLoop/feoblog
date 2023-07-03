{#if shouldLink}
    <a class="userID" class:isPubKey href={link} title={userID.toString()}>{displayName}</a>
{:else}
    <span class="userID" class:isPubKey title={userID.toString()}>{displayName}</span>
{/if}


<script lang="ts">
import { getContext } from "svelte";

import type { Writable } from "svelte/store";
import type { AppState } from "../ts/app";
import type { UserID } from "../ts/client";

export let userID: UserID
// Whether we should resolve an ID into its displayName
export let resolve = true

let appState: Writable<AppState> = getContext("appStateStore")

export let displayName: string = ""
export let href: string|undefined = undefined
export let shouldLink = true

$: defaultHref = $appState.navigator.userPosts(userID).hash
$: link = href || defaultHref

// The display name is actually the public key until we fetch the real name:
let isPubKey = true

$: {
    userID || $appState
    fetchDisplayName()
}

async function fetchDisplayName() {
    // Placeholder value while we fetch things:
    if (!displayName) {
        displayName = userID.toString()
        isPubKey = true
    }

    if (!resolve) return

    let preferredName = await $appState.getPreferredName(userID)
    if (preferredName) {
        displayName = preferredName
        isPubKey = false
    }
}

</script>

<style>
.isPubKey {
    font-family: Consolas, monospace;
    font-weight: normal;
}

.userID {
    font-weight: bold;
}
</style>