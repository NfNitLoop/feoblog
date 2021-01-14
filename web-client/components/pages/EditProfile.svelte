<div class="dualPaneEditor">
    <div class="item editPane">
        <h1><input type="text" bind:value={displayName} disabled={!editable} placeholder="(Profile Display Name)"></h1>
        <div class="userInfo">
            <span class="userID">@{userID}</span>
        </div>
        <ExpandingTextarea bind:value={profileContent} placeholder="Your profile here..." disabled={!editable}/>

        <h2>Follows</h2>
        {#each follows as follow, index (follow)} 
            <FollowBox 
                bind:userID={follows[index].userID} 
                bind:displayName={follows[index].displayName}
                on:delete={() => removeFollow(index)}
            />
        {/each}
        <Button on:click={addFollow} disabled={!editable}>New Follow</Button>
    </div>

    <ItemView
        userID={userID.toString()}
        signature="unknown"
        item={itemProto}
        linkMode="stay"
    />

    <div class="item sendBox inputWhiteBox">
        {#if validationErrors.length > 0}
            <div class="error">
                {#each validationErrors as error}
                    {error}<br>
                {/each}
            </div>
        
        {:else if !validSignature}
            <InputBox 
                inputType="password"
                label="Private Key"
                bind:value={privateKey}
                bind:errorMessage={privateKeyError}
                disabled={!editable}
            />
            <Button on:click={sign} disabled={!validPrivateKey}>Sign</Button>
        {:else}
            <InputBox
                label="Signature"
                value={signature}
                disabled={true}
             />
            <div class="buttons">
                <Button on:click={submit}>Submit</Button>
            </div>
            {#if status}
                <div>{status}</div>
            {/if}
        {/if}
    </div>

</div>




<script lang="ts">
import { onMount, tick } from 'svelte'
import type { Writable } from "svelte/store"
import bs58 from "bs58"
import * as commonmark from "commonmark"
import moment from "moment"
import { Follow, Item, Post, Profile, UserID } from "../../protos/feoblog"
import * as nacl from "tweetnacl-ts"
import bs58check from 'bs58check';
import FollowBox from "../FollowBox.svelte"
import { MAX_ITEM_SIZE, parseUserID } from '../../ts/common'
import { UserID as ClientUserID } from "../../ts/client"
import type { AppState } from '../../ts/app';
import ItemView from '../ItemView.svelte'
import ExpandingTextarea from '../ExpandingTextarea.svelte'
import LinkIntercept from '../LinkIntercept.svelte';
import Button from '../Button.svelte'
import InputBox from '../InputBox.svelte';

export let appState: Writable<AppState>
let userID: ClientUserID
$: userID = function() {
    let userID = $appState.loggedInUser
    if (userID) return userID
    throw `Must be logged in.`
}()

enum PageState {
    // Loading the latest profile.
    Loading,
    Editing,
    Signed,
    // Sent -> Editing
}

let pageState = PageState.Loading
$: editable = (pageState == PageState.Editing)

onMount(() => {
    loadProfile()
})

async function loadProfile() {
    if (!userID) { return }
    let result = await $appState.client.getLatestProfile(userID)
    if (result) {
        let profile = result.item
        loadFromProto(profile)
        signature = result.signature.toString()
    }
    pageState = PageState.Editing
    

}

const reader = new commonmark.Parser()
const writer = new commonmark.HtmlRenderer({ safe: true})

// Strictly parse one of these non-ambiguous timestamps. (MUST include time zone.)
const DATE_FORMATS = [
    // Preferred:
    "YYYY-MM-DD HH:mm:ss.SSS ZZ",
    // May drop milliseconds:
    "YYYY-MM-DD HH:mm:ss ZZ",
    // ... and seconds:
    "YYYY-MM-DD HH:mm ZZ",
]

let displayName = ""
let profileContent = ""
let status = ""




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

let privateKey = ""

// TODO: Move parsing a private key to a separate function and component.
// Error to display about the private key:
$: privateKeyError = function() {
    if (privateKey.length == 0) {
        return "";
    }
    
    let buf: Uint8Array;
    try {
        buf = bs58.decode(privateKey)
    } catch (error) {
        return "Not valid base58"
    }

    // Secret is 32 bytes, + 4 for checked base58.
    if (buf.length < 36) {
        return "Password is too short."
    }
    if (buf.length > 36) {
        return "Password is too long."
    }

    try {
        buf = bs58check.decode(privateKey)
    } catch (e) {
        return "Invalid Password"
    }

    
    let keypair = nacl.sign_keyPair_fromSeed(buf);
    
    let pubKey = bs58.encode(keypair.publicKey)
    if (pubKey != userID.toString()) {
        return "Private key does not match user ID."
    }

    return ""    
}()

// We have a key which could be used to sign.
$: validPrivateKey = privateKey.length > 0 && privateKeyError == ""

let signature = ""

let debug = false
$: {
    if (profileContent.startsWith("!!!debug")) {
        debug = true
    } else if (profileContent.startsWith("!!!nodebug")) {
        debug = false
    }
}

$: markdownOut = function() {
    var parsed = reader.parse(profileContent);
    return writer.render(parsed);
}()


// <3 Moment in that it'll keep the time and offset together:
// TODO: Only save when signed:
let timestampMoment = moment()

function updateTimestmap() {
    timestampMoment = moment()
}

// Used for display in the rendered post.
// TODO: Deprecate Moment.
$: formattedDate = timestampMoment.format(DATE_FORMATS[0])

// Note, is not necessarily a valid Item to send.
// ex: a follow.user.bytes may be empty, though it is required.
$: itemProto = function(): Item {
    let item = new Item({
        timestamp_ms_utc: timestampMoment.valueOf(),
        utc_offset_minutes: timestampMoment.utcOffset(),
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

    // TODO: servers.

    return item

}()

// This is the inverse of $: itemProto above. Given an Item, load data from it.
function loadFromProto(item: Item) {
    let profile = item.profile
    timestampMoment = moment.utc(item.timestamp_ms_utc).utcOffset(item.utc_offset_minutes)
    displayName = profile.display_name
    profileContent = profile.about

    let _follows = new Array<FollowEntry>()
    profile.follows.forEach((follow) => {
        let f = new FollowEntry(ClientUserID.fromBytes(follow.user.bytes).toString(), follow.display_name)
        _follows.push(f)
    })

    follows = _follows

    // TODO: servers
}

$: itemProtoBytes = itemProto.serialize()
$: protoSize = itemProtoBytes?.length || 0
$: protoHex = debug ? bufferToHex(itemProtoBytes || []) : ""

$: itemJson = JSON.stringify(itemProto.toObject(), null, 1)

// Errors that prevent signing:
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

    if (protoSize > MAX_ITEM_SIZE) {
        errs.push(`Item size is ${protoSize}/${MAX_ITEM_SIZE}`)
    }

    return errs
}()

$: readyToSign = validationErrors.length == 0



$: validSignature = function(): boolean {
    if (!userID || !signature || !itemProtoBytes) {
        return false
    }
    try {
        let pubKey = userID.bytes
        let decodedSig = bs58.decode(signature)
        let ok = nacl.sign_detached_verify(itemProtoBytes, decodedSig, pubKey)
        return ok;
    } catch (error) {
        console.error("Error validating signature:", error)
        return false
    }
}()


function bufferToHex (x: Iterable<number>) {
    return [...new Uint8Array (x)]
        .map (b => b.toString(16).padStart(2, "0"))
        .join (" ");
}

// Create a signature, delete the password.
async function sign() {

    updateTimestmap()
    await tick()

    if (privateKeyError) {
        console.error("Shouldn't be able to call sign w/ invalid private key.")
        return
    }

    if (!itemProtoBytes) throw `No bytes to sign.`
   
    let buf = bs58check.decode(privateKey)
    let keypair = nacl.sign_keyPair_fromSeed(buf);
    let binSignature = nacl.sign_detached(itemProtoBytes, keypair.secretKey)
    signature = bs58.encode(binSignature)

    // Delete the privateKey, we don't want to save it any longer than
    // necessary:
    privateKey = ""
}

function unSign() {
    signature = ""
}

async function submit() {
    if (!readyToSign || !validSignature) {
        console.error("Submit clicked when not valid");
        return;
    }

    let url = `/u/${userID}/i/${signature}/proto3`
    let bytes = itemProtoBytes;
    status = "Making request"
    
    let response: Response
    try {
        response = await fetch(url, {
            method: "PUT",
            body: bytes,
        })
    } catch (e) {
        console.error("PUT exception:", e)
        status = `PUT exception: ${e}`
        return 
    }

    if (debug) {
        console.debug("response:")
        console.debug(response)
    }

    let code = response.status
    let message = await response.text()
    status = `${code}: ${message}`
}

</script>

<style>
.buttons {
    margin-top: 1em;
}
</style>