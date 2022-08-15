<PageHeading />

<div class="item">
<div class="header">
    <div class="text">Sync</div>
    <OpenArrow bind:isOpen={settingsOpen}/>
</div>

{#if settingsOpen}
<div transition:slide|local class="inset">

    <div class="section">
        <select bind:value={syncType} disabled={settingsDisabled}>
            <option value="n">Sync N items from each user</option>
            <option value="days">Sync N days from each user</option>
            <option value="all">Sync all history from each user</option>
        </select>

        {#if syncType=="n"}
            <p>This mode is best if you just want to have some items in your feed to read from everyone you follow.
                It makes sure you have items from both those that post often, and those who post very rarely.
            </p>

            <InputBox
                label="Maximum Posts Per User"
                bind:value={syncCount}
                validationCallback={intBetween(0, Number.POSITIVE_INFINITY)}
                bind:errorMessage={syncCountError}
                disabled={settingsDisabled}
            />
        {:else if syncType == "days"}
            <p>Because this mode only syncs back a certain number of days, you might not get any posts from those that don't post often.</p>

            <InputBox
                label="Maximum Number of Days"
                bind:value={syncDays}
                validationCallback={intBetween(0, Number.POSITIVE_INFINITY)}
                bind:errorMessage={syncDaysError}
                disabled={settingsDisabled}
            />
        {/if}
    </div>

    <OptionalSection buttonText="Specify Server" bind:checked={specifyServer} disabled={settingsDisabled}>
        <p>Disregard the servers listed in users' profiles and only pull from a single server.
            This is useful when performing an initial sync, when you might not yet have users' profiles (which list which servers to
            pull from) available.
        </p>

        <InputBox 
            label="Server URL"
            placeholder="(Optional. Default: Servers listed in profiles)"
            validationCallback={validateServerURL}
            disabled={settingsDisabled}
            bind:errorMessage={serverURLError}
            bind:value={serverURL}
        />
    </OptionalSection>

    <OptionalSection buttonText="Backfill Attachments" bind:checked={backfillAttachments} disabled={settingsDisabled}>
        <p>For items already on the server, do extra work to double check that their attachments also have been copied.</p>
        <p>Usually, when syncing an item, attachments are copied. But that can fail for several reasons. This will attempt to
            re-try copying attachments that are missing.
        </p>
        <p>As of now, this attempts to backfill attachments for ALL Items already on the server.</p>
        <p>TODO: Limit it to the same time span as configured above.</p>
    </OptionalSection>


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
            disabled={startButtonsDisabled}
        >Pull My Feed</Button>

        <Button
            on:click={publishMyPosts}
            disabled={startButtonsDisabled}
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

.section {
    margin: 1rem 0rem;
}


</style>

<script lang="ts">
import { getContext } from "svelte";
import type { Writable } from "svelte/store"
import { slide } from "svelte/transition"
import type { AppState } from "../../ts/app"
import type { UserID } from "../../ts/client"
import { TaskTracker, validateServerURL } from "../../ts/common";
import Button from "../Button.svelte"
import OpenArrow from "../OpenArrow.svelte"
import InputBox from "../InputBox.svelte"
import TaskTrackerView from "../TaskTrackerView.svelte";
import PageHeading from "../PageHeading.svelte";
import {  publishMyPostsTask,  syncMyFeedTask, SyncOptions } from "../../ts/sync";
import OptionalSection from "../widgetes/OptionalSection.svelte";
import { DateTime } from "luxon";

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

let specifyServer = false
let serverURL = ""
let serverURLError = ""
let backfillAttachments = false;

let syncType: "n"|"days"|"all" = "n"
let syncCount = "50"
let syncCountError = ""
let syncDays = "7"
let syncDaysError = ""

let taskTracker = new TaskTracker()


$: errors = function() {
    let errs: string[] = []
    let check = (prereq: boolean, error: string|null) => {
        if (prereq && error) errs.push(error)
    }

    check(syncType == "n", syncCountError)
    check(syncType == "days",  syncDaysError)
    check(specifyServer, serverURLError)

    return errs
}()

$: settingsDisabled = taskTracker.isRunning
$: startButtonsDisabled = taskTracker.isRunning || errors.length > 0


const ONE_DAY_MS = 24 * 60 * 60 * 1000

function getSyncOpts(): SyncOptions|null {
    if (errors.length > 0) {
        return null
    }

    let opts: SyncOptions = {
        localClient: $appState.client,
    }

    if (specifyServer) {
        opts.sourceServerUrl = serverURL
    }

    if (syncType == "n") {
        opts.recentItems = Number.parseInt(syncCount)
    } else if (syncType == "days") {
        let numDays = Number.parseInt(syncDays)
        opts.toDateUtcMs = DateTime.utc().valueOf() - (numDays * ONE_DAY_MS)
    }

    if (backfillAttachments) {
        opts.backfill = {
            // TODO: Specify how many bytes of attachments to copy, max file size, etc, etc.
        }
    }

    return opts
}


async function syncMyFeed() {

    const opts = getSyncOpts()
    if (opts == null) {
        console.error("syncMyFeed called while there were errors", errors)
        return
    }

    let result = await taskTracker.run("Syncing my feed", (tracker) => syncMyFeedTask(tracker, userID, opts))

    if (result.updatedUserProfile) {
        $appState.userProfileChanged()
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


// Create a validator for integers:
function intBetween(lowValue: number, highValue: number): (value: string) => string {
    let pat = /^-?[0-9]+$/

    return (value: string) => {

        if (!value.match(pat)) {
            return "Not a valid number"
        }


        let parsed: number
        try {
            parsed = Number.parseInt(value)
        } catch (error: unknown) {
            return `Not a valid integer: ${error}`
        }

        if (parsed < lowValue) {
            return `Value must be > ${lowValue}`
        }
        if (parsed > highValue) {
            return `Value must be < ${highValue}`
        }

        return ""
    }
}


</script>