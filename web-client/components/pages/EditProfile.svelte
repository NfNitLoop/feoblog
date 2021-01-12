<!-- TODO: rename "postPage", responsible for side-by-side view  -->
<div id="postPage">
    <div class="postInput item">

        <table>
            <tr>
                <th><label for="userID">User ID</label>:</th>
                <td><input class="userID" type="text" name="userID" value={userID.toString()} disabled>
                </td>
            </tr>
            <tr>
                <th>Display&nbsp;Name:</th>
                <td><input type="text" bind:value={displayName} disabled={!editable}></td>
            </tr>
            <tr>
                <td colspan="2">
                    <textarea bind:this={textbox} bind:value={profileContent} placeholder="Your profile here..." disabled={!editable}></textarea>
                </td>
            </tr>
            <tr>
                <td colspan="2">
                    <hr>
                    <h2>Following {follows.length} Users:</h2>
                    {#each follows as follow, index (follow)} 
                        <FollowBox 
                            bind:userID={follows[index].userID} 
                            bind:displayName={follows[index].displayName}
                            on:delete={() => removeFollow(index)}
                        />
                    {/each}
                    <button on:click={addFollow} disabled={!editable}>Add</button>
                </td>
            </tr>
        </table>
    </div>


    <div class="postPreview item" bind:this={postPreviewDiv}>
        {#if displayName}
            <h1 class="title">{ displayName }</h1>
        {/if}
        <div><span class="userID">@{userID}</span></div>
        
        {@html markdownOut}

        <hr>

        {#if follows.length > 0}
            <h2>Follows {follows.length} users</h2>
            <ul>
            {#each follows as follow}
                {#if follow.displayName} 
                    <li><a href="/u/{follow.userID}">{follow.displayName}</a></li>    
                {:else}
                    <li><a href="/u/{follow.userID}" class="userID">@{follow.userID}</a></li>    

                {/if}
            {/each}
            </ul>
        {/if}

        <div class="timestamp">Last updated: { formattedDate }</div>

    </div>

    <div class="item sendBox">
        <table>
            {#if validationErrors.length > 0}
            <tr>
                <th></th>
                <td class="error">
                    {#each validationErrors as error}
                        {error}<br>
                    {/each}
                </td>
            </tr>

            {:else if !validSignature}
            <tr>
                <th><label for="privateKey">Private Key</label>:</th>
                <td>
                    <input type="password" name="privateKey" bind:value={privateKey} disabled={!editable}>
                    {#if privateKeyError}
                    <div class="error">{privateKeyError}</div>
                    {/if}
                </td>
            </tr>
            <tr>
                <th></th>
                <td><button name="sign" on:click={sign} disabled={!validPrivateKey}>Sign</button></td>
            </tr>
            {:else}
            <tr>
                <th><label for="signature">Signature</label>:</th>
                <td><input type="text" name="signature" class="signature" bind:value={signature} disabled></td>
            </tr>
            <tr>
                <th></th>
                <td>
                    <button name="submit" on:click={submit}>Submit</button>
                    {#if status}
                        <div>{status}</div>
                    {/if}
                </td>
            </tr>
            {/if}
        </table>
    </div>
    

</div>



{#if debug}
<div class="protoPreview">
    <pre>
bytes: {protoSize}
{itemJson}
    </pre>

    binary: <code>{ protoHex }</code>
</div>
{/if}

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
let textbox
let postPreviewDiv: HTMLElement
let status = ""
onMount(() => {
    // <textarea>:
    textbox.focus()
    textbox.selectionStart = 0
    textbox.selectionEnd = textbox.value.length
    

    postPreviewDiv.onclick = interceptLinkClicks
})

// TODO: Make this a reusable widget.
// Send link clicks to target=_blank to save the contents of the edit box:
function interceptLinkClicks(event: Event) {
    let target = event.target as HTMLElement
    let anchor: HTMLAnchorElement | null = null
    let tag = target.tagName

    if (tag == "A") {
        anchor = (target as HTMLAnchorElement)
    } else if (tag == "IMG") {
        let parent = target.parentElement
        if (parent?.tagName == "A") {
            anchor = (parent as HTMLAnchorElement)
        }
    }

    if (!anchor) { return }
    anchor.target = "_blank"
}

$: {
    profileContent // on change:
    // TODO: Make a reusable expanding textarea widget.
    expandTextarea(textbox)
}

function expandTextarea(textarea) {
    if (!textarea) { return } // not mounted yet
    
    if (textarea.scrollHeight > textarea.clientHeight) {
        let borderHeight = textarea.offsetHeight - textarea.clientHeight
        textarea.style.height = textarea.scrollHeight + borderHeight;
    }
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

<style type="text/css">
    @media (min-width: 60em) {
        #postPage {
            display: inline-grid;
            width: 100%;
            /* a single items has max-width 55em. +1em grid gap */
            max-width: 111em;
            grid-template-columns: 1fr 1fr;
            grid-gap: 1em;
            padding: 1em;
        }
        #postPage > * {
            margin: 0px;
        }
    }
   
    input {
        width: 100%;
    }

    textarea {
        margin-top: 1em;
        border: 0px;
        min-height: 20em;
        width: 100%;
    }
       
    table {
        width: 100%;
    }
    table th {
        text-align: right;
        width: auto;
        min-width: 12ch;
        vertical-align: top;
    }
    table td {
        width: 100%;
        vertical-align: top;
    }
    
    .error {
        color: red;
        font-weight: bold;
    }
    
    .protoPreview {
        overflow: hidden;
    }
    
    </style>
    