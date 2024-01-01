<!-- 
    View a user's profile.

    If the user == the logged-in user, allow editing, too.
-->

<PageHeading />

{#if error || !userID}
    <div class="item"><div class="body error"><p>{error}</p></div></div>
{:else if loading}
    <div class="item"><div class="body">Loading...</div></div>
{:else if !profile && !ownProfile}
    <ItemBox>
        <p>No profile for user <UserIDView {userID} shouldLink={false}/>
    </ItemBox>
{:else}
    {#if ownProfile}
        <TabBar {tabs} bind:activeTab/>
    {/if}
    {#if activeTab == "Current"}
        {#if profile}
            <ItemView 
                item={profile.item}
                userID={userID.toString()}
                signature={profile.signature.toString()}
            />
        {:else}
            <div class="item"><div class="body">
                <p>No profile found. Use the "Edit" tab above to create one!</p>
            </div></div>
        {/if}
    {:else if mutableProfile}
        <EditorWithPreview bind:item={mutableProfile} reuse mode="profile" {activeTab}/>
    {/if}

{/if}



<script lang="ts">
import type { Writable } from "svelte/store";
import type { AppState } from "../../ts/app";

import { getContext } from "svelte";
import { params } from "svelte-hash-router"

import ItemView from "../ItemView.svelte";
import PageHeading from "../PageHeading.svelte";
import TabBar from "../TabBar.svelte";
import EditorWithPreview from "../EditorWithPreview.svelte";
import ItemBox from "../ItemBox.svelte";
import UserIDView from "../UserIDView.svelte"
import { protobuf as pb, ProfileResult, UserID } from "feoblog-client"
import { ConsoleLogger } from "../../ts/common";

const logger = new ConsoleLogger({prefix: "<ProfilePage>"})
logger.debug("loaded")


let appState: Writable<AppState> = getContext("appStateStore")

$: userID = UserID.tryFromString($params.userID)
$: loadProfile(userID)
$: loggedInUser = $appState.loggedInUser

// user is viewing their own profile
$: ownProfile = userID != null && loggedInUser?.asBase58 == userID.asBase58

let profile: ProfileResult|null = null
let mutableProfile: pb.Item | null = null

let error: string|null = null
let missingProfile = false
let loading = true

type TabType = "Current" | "Edit" | "Preview"
let tabs: TabType[] = ["Current", "Edit", "Preview"]
let activeTab: TabType = "Current"


async function loadProfile(userID: UserID|null): Promise<void> {
    logger.log("loadProfile()")
    profile = null
    missingProfile = false
    error = null
    loading = false

    if (!userID) {
        error = `UserID is required`
        return
    }

    loading = true
    try {
        // Note: always load from server, not the appState cache.
        // User may want to see changes in the profile.
        profile = await $appState.client.getProfile(userID)
        missingProfile = profile == null
        mutableProfile = profile ? profile.item.clone() : new pb.Item()
    } finally {
        loading = false
    }
}



</script>