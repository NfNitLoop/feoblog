<PageHeading />

<div class="item">
<div class="header">
    <div class="text">Sync</div>
    <OpenArrow bind:isOpen={settingsOpen}/>
</div>

{#if settingsOpen}
<div transition:slide|local class="inset">
    <InputBox 
        label="Server URL"
        placeholder="(Optional. Default: Servers listed in profiles)"
        validationCallback={validateServerURL}
        disabled={taskTracker.isRunning}
        bind:errorMessage={serverURLError}
        bind:value={serverURL}
    />

    <CheckBox disabled={taskTracker.isRunning} onGrey bind:checked={backfillAttachments}>Backfill Attachments</CheckBox>

</div>
{/if}

<div class="body">
    <p>Synchronize your posts (and your feed) from/to other servers</p>
    <ul>
        <li><strong>Pull My Feed</strong>: Copies posts of those you follow from their various servers onto this one.</li>
        <li><strong>Push My Posts</strong>: Copies posts you've made on this server to any servers you've listed in your profile.</li>
    </ul>

    <div class="buttons">
        <Button 
            on:click={syncMyFeed} 
            disabled={taskTracker.isRunning || serverURLError != ""}
        >Pull My Feed</Button>

        <Button
            on:click={publishMyPosts}
            disabled={taskTracker.isRunning || serverURLError != ""}
        >Push My Posts</Button>
    </div>

    <TaskTrackerView bind:tracker={taskTracker}/>
</div>
</div>

<style>

.text {
    font-weight: bold;
    /* margin-left: 0.5em; */
    padding: 0.5em 0;
}

.buttons {
    margin-bottom: 1em;
}


</style>

<script lang="ts">
import { getContext } from "svelte";
import type { Writable } from "svelte/store"
import { slide } from "svelte/transition"
import type { ItemListEntry, Profile } from "../../protos/feoblog";
import { Item } from "../../protos/feoblog"
import type { AppState } from "../../ts/app"
import type { AttachmentMeta, ProfileResult, UserID } from "../../ts/client"
import { bytesToHex, prefetch, readableSize, TaskTracker, validateServerURL } from "../../ts/common";
import Button from "../Button.svelte"
import CheckBox from "../CheckBox.svelte"
import OpenArrow from "../OpenArrow.svelte"
import InputBox from "../InputBox.svelte"
import TaskTrackerView from "../TaskTrackerView.svelte";
import PageHeading from "../PageHeading.svelte";
import { entryByTimestampSigDesc, publishMyPostsTask, serversFromProfile, syncMyFeedTask, SyncOptions, SyncUserArgs, syncUserProfile } from "../../ts/sync";

// TODO: Backfill!
// TODO: A button to stop a sync.
// TODO: Stop the sync when you go to another tab.
// TODO: If logged-in user has no profile, just preempt asking for URL for a sync (and disable sync w/o it)

let appState: Writable<AppState> = getContext("appStateStore")

let settingsOpen = false

let userID: UserID
$: userID = function() {
    let id = $appState.loggedInUser
    if (!id) throw `Must be logged in`
    return id
}()

$: breadcrumbs = getBreadcrumbs(userID)

function getBreadcrumbs(userID: UserID|null) {
    let crumbs = []
    if (userID) {
        crumbs.push({userID})
    }

    crumbs.push({text: "Sync"})

    return {crumbs}
}


let serverURL = ""
let serverURLError = ""
let backfillAttachments = false;

let taskTracker = new TaskTracker()



async function syncMyFeed() {

    const opts: SyncOptions = {
        localClient: $appState.client,
        sourceServerUrl: serverURL,

        recentItems: 50, // TODO: Configurable by user.
        // TODO: backfill: { ... }
    }


    let result = await taskTracker.run("Syncing my feed", (tracker) => syncMyFeedTask(tracker, userID, opts))

    if (result.updatedUserProfile) {
        // Let appState know.
    }
}

function publishMyPosts() {
    taskTracker.run("Publish my posts", (tracker) => publishMyPostsTask({
        tracker,
        userID,
        local: $appState.client,
        serverURL
    }))
}



</script>