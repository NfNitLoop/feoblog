<a class="userID" href={`#/u/${userID}/`}>@{displayName}</a>

<script lang="ts">
import type { Writable } from "svelte/store";
import type { AppState } from "../ts/app";
import type { UserID } from "../ts/client";

export let userID: UserID
export let appState: Writable<AppState>

// May resolve to a preferred display name:
let namePromise: Promise<string|null> = Promise.resolve(null)

$: {
    userID || appState
    fetchDisplayName()
}
$: displayName = userID.toString()

async function fetchDisplayName() {
    // Placeholder value while we fetch things:
    displayName = userID.toString()

    let preferredName = await $appState.getPreferredName(userID)
    if (preferredName) {
        displayName = preferredName
    }
}

</script>