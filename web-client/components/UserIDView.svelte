<a class="userID" href={link}>@{displayName}</a>

<script lang="ts">
import type { Writable } from "svelte/store";
import type { AppState } from "../ts/app";
import type { UserID } from "../ts/client";

export let userID: UserID
// Whether we should resolve an ID into its displayName
export let resolve = true
export let appState: Writable<AppState>
export let displayName: string
export let href: string|undefined

$: defaultHref = `#/u/${userID}/`
$: link = href || defaultHref

$: {
    userID || appState
    fetchDisplayName()
}

async function fetchDisplayName() {
    // Placeholder value while we fetch things:
    if (!displayName) displayName = userID.toString()

    if (!resolve) return

    let preferredName = await $appState.getPreferredName(userID)
    if (preferredName) {
        displayName = preferredName
    }
}

</script>