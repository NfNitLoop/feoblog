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
import { Follow, Item, Profile, Server, UserID } from "../protos/feoblog";
import type { AppState } from "../ts/app";
import type { Writable } from "svelte/store";
import { UserID as ClientUserID } from "../ts/client";
import { parseUserID, validateServerURL } from "../ts/common";
import { tick } from "svelte";
import bs58 from "bs58";
import InputBox from "./InputBox.svelte";
import { DateTime } from "luxon";

export let appState: Writable<AppState>
// Exported so that EditorWithPreview can preview, serialize, & send it for us.
export let item: Item

// Possibly imported, so we can start editing an existing profile:
export let initialItem: Item|undefined

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

    toFollow(): Follow {
        return new Follow({
            display_name: this.displayName,
            user: new UserID({
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

// This is the inverse of $: itemProto above. Given an Item, load data from it.
function loadFromProto(item: Item) {
    let profile = item.profile
    displayName = profile.display_name
    profileContent = profile.about

    let _follows = new Array<FollowEntry>()
    profile.follows.forEach((follow) => {
        let f = new FollowEntry(ClientUserID.fromBytes(follow.user.bytes).toString(), follow.display_name)
        _follows.push(f)
    })

    follows = _follows

    servers = profile.servers.map((s) => { return {url: s.url} })
}

if (initialItem) {
    loadFromProto(initialItem)
}


$: item = function(): Item {
    // For profiles, we *always* want to save with the latest timestamp possible:
    let now = DateTime.local()

    let item = new Item({
        timestamp_ms_utc: now.valueOf(),
        utc_offset_minutes: now.offset,
        profile: new Profile({
            display_name: displayName,
            about: profileContent,
        })
    })

    let profile = item.profile
    follows.forEach(entry => {
        let userIDBytes = new Uint8Array()
        try {
            let buf: Buffer = bs58.decode(entry.userID)
            // While a Buffer in theory extends a Uint8Array, the google-protobuf library
            // checks the constuctor of the object to make sure it's actually a Uint8Array.
            // See: https://github.com/protocolbuffers/protobuf/issues/1319
            userIDBytes = new Uint8Array(buf)
        } catch (_ignored) {}

        profile.follows.push(new Follow({
            user: new UserID({bytes: userIDBytes}),
            display_name: entry.displayName,
        }))
    })

    for (let {url} of servers) {
        if (url === "") continue
        profile.servers.push(new Server({url}))
    }

    return item
}()



</script>
