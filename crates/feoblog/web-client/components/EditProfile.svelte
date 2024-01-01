<!--
    Component for editing a profile. Used from within EditorWithPreview
-->
<div class="item editPane">
    <div class="body">
        <h1><input type="text" bind:value={displayName} placeholder="(Profile Display Name)"></h1>
        <ExpandingTextarea bind:value={profileContent} placeholder="Your profile here..."/>

        <h2>Follows</h2>
        {#each follows as follow, index (follow)} 
            <FollowBox 
                bind:userID={follows[index].userID} 
                bind:displayName={follows[index].displayName}
                on:delete={() => removeFollow(index)}
            />
        {/each}
        <Button on:click={addFollow}>New Follow</Button>


        <h2>Servers</h2>
        {#each servers as server, index (server)}
            <div class="inputsGreyBox">
            <InputBox
                label="Server"
                placeholder="https://feoblog.example.com"
                validationCallback={validateServerURL}
                bind:value={servers[index].url}
            />
            </div>
        {/each}
    </div>
</div>


<script lang="ts">
import ExpandingTextarea from "./ExpandingTextarea.svelte"
import FollowBox from "./FollowBox.svelte"
import Button from "./Button.svelte"
import type { AppState } from "../ts/app";
import type { Writable } from "svelte/store";
import { UserID as ClientUserID, protobuf as pb, getInner } from "../ts/client";
import { parseUserID, validateServerURL } from "../ts/common";
import { getContext, onMount, tick } from "svelte";
import InputBox from "./InputBox.svelte";
import { DateTime } from "luxon";
import { decodeBase58 } from "../ts/fbBase58";

let appState: Writable<AppState> = getContext("appStateStore")
// Exported so that EditorWithPreview can preview, serialize, & send it for us.
export let item: pb.Item


export let validationErrors: string[] = []
$: validationErrors = function(): string[] {
    let errs = new Array()

    let followErrors = new Set(
        follows.map(f => {
            try {
                f.toFollow()
            } catch (message) {
                return message
            }
            return ""
        }).filter(x => x != "")
    )

    for (let e of followErrors) {
        errs.push(`Follows: ${e}`)
    }

    // Check for duplicate userIDs:
    let map = new Map()
    follows.forEach(follow => {
        let u = follow.userID
        if (!map.has(u)) { map.set(u, 1) }
        else { map.set(u, map.get(u) + 1) }
    })
    for (let entry of map.entries()) {
        let [uid, count] = entry
        if (count > 1) {
            errs.push(`${count} follows for userID ${uid}`)
        }

    }

    for (let {url} of servers) {
        if (url.length == 0) continue
        let error = validateServerURL(url)
        if (error) {
            errs.push(error)
            break // all server errors are the same basically.
        }
    }

    return errs
}()

export let isValid = false
$: isValid = validationErrors.length === 0

let userID: ClientUserID
$: {
    let id = $appState.loggedInUser
    if (!id) throw `Must be logged in.`
    userID = id
}

let displayName = ""
let profileContent = ""

// Which users is this user following?
let follows: FollowEntry[] = []

function removeFollow(index: number) {
    follows.splice(index, 1)
    follows = follows
}

async function addFollow() {
    follows.push(new FollowEntry())
    follows = follows
    await tick()
    // TODO: Focus the new userID box.
}

// A bridge between HTML and the Follow protobuf object.
class FollowEntry {
    userID = ""
    displayName = ""

    constructor(userID = "", displayName = "") {
        this.userID = userID
        this.displayName = displayName
    }

    toFollow(): pb.Follow {
        return new pb.Follow({
            displayName: this.displayName,
            user: new pb.UserID({
                bytes: this.userIDBytes()
            }),
        });
    }
    
    // TODO: Upgrade to a UserID.fromString()
    userIDBytes(): Uint8Array {
        return parseUserID(this.userID)
    }
}

// We use an object here because it can be used as a Svelte #each key.
let servers: {url:string}[] = [{url: ""}]
$: {
    let emptyURLs = servers.filter(s => s.url.length == 0).length
    // Avoid an infinite loop. 😅
    if (emptyURLs != 1) {
        servers = [
            ...servers.filter(s => s.url.length > 0),
            {url: ""}
        ]
    }
}

// This is the inverse of `$: item = ...` below. Given an Item, load data from it.
function loadFromProto(item: pb.Item) {
    let profile = getInner(item, "profile")
    displayName = profile?.displayName ?? ""
    profileContent = profile?.about ?? ""

    let _follows = new Array<FollowEntry>()
    profile?.follows.forEach((follow) => {
        let f = new FollowEntry(ClientUserID.fromBytes(follow.user!.bytes).toString(), follow.displayName)
        _follows.push(f)
    })

    follows = _follows

    servers = profile?.servers.map((s) => { return {url: s.url} }) ?? []
}

loadFromProto(item)


$: item = function(): pb.Item {
    // For profiles, we *always* want to save with the latest timestamp possible:
    let now = DateTime.local()

    let profile = new pb.Profile({
        displayName,
        about: profileContent,
    })

    let item = new pb.Item({
        timestampMsUtc: BigInt(now.valueOf()),
        utcOffsetMinutes: now.offset,
        itemType: { case: "profile", value: profile }
    })

    follows.forEach(entry => {
        let userIDBytes = new Uint8Array()
        try {
            userIDBytes = decodeBase58(entry.userID)
        } catch (_ignored) {}

        profile.follows.push(new pb.Follow({
            user: new pb.UserID({bytes: userIDBytes}),
            displayName: entry.displayName,
        }))
    })

    for (let {url} of servers) {
        if (url === "") continue
        profile.servers.push(new pb.Server({url}))
    }

    return item
}()

</script>
