<div id="postPage">
    <div class="postInput item">
        <h1 class="title"><input type="text" name="title" placeholder="Title (optional)" bind:value={title} disabled={validPost}></h1>
        <div class="timestamp">                    
            <input type="text" name="time" bind:value={timeInput} disabled={validPost}>
            {#if timeInputError}
            <div class="error">{timeInputError}</div>
            {/if}
        </div>
        <textarea bind:this={textbox} bind:value={post} disabled={validPost}></textarea>



        <table class="form">
            <tr>
                <th><label for="userID">User ID</label>:</th>
                <td><input class="userID" type="text" name="userID" bind:value={userID} disabled></td>
            </tr>
            <tr>
                <th><label for="signature">Signature</label>:</th>
                <td><input type="text" name="signature" class="signature" bind:value={signature} disabled></td>
            </tr>
            <tr>
                <th><label for="privateKey">Private Key</label>:</th>
                <td>
                    <input type="password" name="privateKey" bind:value={privateKey}>
                    {#if privateKeyError}
                    <div class="error">{privateKeyError}</div>
                    {/if}

                </td>
            </tr>
            <tr>
                <th></th>
                <td>
                    {#if validPost}
                        <button name="unsign" on:click={unSign}>Edit</button>
                    {:else}
                        <button name="sign" on:click={sign} disabled={!validPrivateKey}>Sign</button>
                    {/if}
                    <button name="submit" on:click={submit} disabled={!validPost}>Submit</button>
                    {#if status}
                        <div>{status}</div>
                    {/if}
                </td>
            </tr>
        </table>
    </div>


    <div class="postPreview item" bind:this={postPreviewDiv}>
        {#if title}
        <h1 class="title">{ title }</h1>
        {/if}
        <div class="timestamp">{ formattedDate }</div>
        
        {@html markdownOut}
    </div>


</div>


<script lang="ts">
import { onMount } from 'svelte';
import bs58 from "bs58"
import * as commonmark from "commonmark"
// TODO: Deprecate and replace with luxon.
import moment from "moment"
import { Item, Post } from "../protos/feoblog"
import * as nacl from "tweetnacl-ts"
import bs58check from 'bs58check';

const reader = new commonmark.Parser()
const writer = new commonmark.HtmlRenderer({ safe: true})
const MAX_ITEM_SIZE = 32 * 1024 // 32k

// Strictly parse one of these non-ambiguous timestamps. (MUST include time zone.)
const DATE_FORMATS = [
    // Preferred:
    "YYYY-MM-DD HH:mm:ss.SSS ZZ",
    // May drop milliseconds:
    "YYYY-MM-DD HH:mm:ss ZZ",
    // ... and seconds:
    "YYYY-MM-DD HH:mm ZZ",
]

let title = ""
let post = `Hello **world**!

Welcome to [FeoBlog].

[FeoBlog]: https://www.github.com/nfnitloop/feoblog/
`
let textbox // .value holds `post`
let postPreviewDiv: HTMLElement
let status = ""
onMount(() => {
    // <textarea>:
    textbox.focus();
    textbox.selectionStart = 0;
    textbox.selectionEnd = textbox.value.length;

    // // contenteditable element:
    // let range = document.createRange()
    // range.selectNode(textbox)
    // let sel = window.getSelection()
    // sel.removeAllRanges()
    // sel.addRange(range)
    // textbox.focus()

    postPreviewDiv.onclick = interceptLinkClicks
})
// focusTextBox()

// Send link clicks to target=_blank to save the contents of the edit box:
function interceptLinkClicks(event: Event) {
    let target = event.target as HTMLElement
    let anchor: HTMLAnchorElement = undefined
    let tag = target.tagName

    if (tag == "A") {
        anchor = (target as HTMLAnchorElement)
    } else if (tag == "IMG") {
        let parent = target.parentElement
        if (parent.tagName == "A") {
            anchor = (parent as HTMLAnchorElement)
        }
    }

    if (!anchor) { return }
    anchor.target = "_blank"
}

$: {
    post // on change:
    expandTextarea(textbox)
}

function expandTextarea(textarea) {
    if (!textarea) { return } // not mounted yet
    
    if (textarea.scrollHeight > textarea.clientHeight) {
        let borderHeight = textarea.offsetHeight - textarea.clientHeight
        textarea.style.height = textarea.scrollHeight + borderHeight;
    }
}


// <3 Moment in that it'll keep the time and offset together:
let timestampMoment = moment()

let timeInput = timestampMoment.format(DATE_FORMATS[0])
$: {
    if (!timeInputError) {
        timestampMoment = parseDate(timeInput)
    }
}

$: timeInputError = function() {
    let date = parseDate(timeInput)
    if (!date.isValid()) {
        return "Invalid date format."
    }

    let now = moment()
    if (date.valueOf() > now.valueOf()) {
        return "Date is in the future."
    }
}()

let userID = ""
let privateKey = ""

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
    userID = bs58.encode(keypair.publicKey)

    return ""    
}()

$: validPrivateKey = privateKey.length > 0 && privateKeyError == ""

let signature = ""

let debug = false
$: {
    if (post.startsWith("!!!debug")) {
        debug = true
    } else if (post.startsWith("!!!nodebug")) {
        debug = false
    }
}

function parseDate(str: string): moment.Moment {
    let date: moment.Moment;
    for (let i in DATE_FORMATS) {
        // keep the parsed offset in the Moment so we can render/save it.
        date = moment.parseZone(str, DATE_FORMATS[i], true)
        if (date.isValid()) {
            return date
        }
    }
    return date;
}

$: markdownOut = function() {
    var parsed = reader.parse(post);
    return writer.render(parsed);
}()

// Used for display in the rendered post.
$: formattedDate = timestampMoment.format(DATE_FORMATS[0])


$: itemProto = function(): Item {
    return new Item({
        timestamp_ms_utc: timestampMoment.valueOf(),
        utc_offset_minutes: timestampMoment.utcOffset(),
        post: new Post({
            title: title,
            body: post,
        })
    })
}()

$: itemProtoBytes = itemProto.serialize()
$: protoSize = itemProtoBytes.length
$: protoHex = debug ? bufferToHex(itemProtoBytes) : "";

$: itemJson = JSON.stringify(itemProto.toObject(), null, 1)

$: errors = function(): string[] {
    let errs = new Array()
    if (!userID) {
        errs.push("Must sign the message")
    }

    if (protoSize > MAX_ITEM_SIZE) {
        errs.push(`Item size is ${protoSize}/${MAX_ITEM_SIZE}`)
    }

    if (timeInputError) {
        errs.push(timeInputError)
    }

    if (!validSignature) {
        errs.push("Invalid Signature")
    }

    return errs
}()

// This post is valid and signed and ready to send to the server:
$: validPost = errors.length == 0;

$: validSignature = function(): boolean {
    if (!userID || !signature) {
        return false
    }
    try {
        let pubKey = bs58.decode(userID)
        let decodedSig = bs58.decode(signature)
        let ok = nacl.sign_detached_verify(itemProtoBytes, decodedSig, pubKey)
        return ok;
    } catch (error) {
        console.error("Error validating signature:", error)
        return false
    }
}()


function bufferToHex (x) {
    return [...new Uint8Array (x)]
        .map (b => b.toString(16).padStart(2, "0"))
        .join (" ");
}

// Create a signature, delete the password.
function sign() {
    if (privateKeyError) {
        console.error("Shouldn't be able to call sign w/ invalid private key.")
        return
    }
   
    let buf = bs58check.decode(privateKey)
    let keypair = nacl.sign_keyPair_fromSeed(buf);
    let binSignature = nacl.sign_detached(itemProtoBytes, keypair.secretKey)
    signature = bs58.encode(binSignature)

    // Delete the privateKey, we don't want to save it any longer than
    // necessary:
    privateKey = ""

    console.log("generated signature", signature)
}

function unSign() {
    signature = ""
}

async function submit() {
    if (!validPost) {
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
        console.log("PUT exception:", e)
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
            display: grid;
            width: 100%;
            grid-template-columns: 1fr 1fr;
            /* max-height: 80vh; */
        }
        #postPage :first-child {
            margin-right: 0px;
        }
    }

    .title input, .timestamp input {
        font-size: inherit;
        font-family: inherit;
        font-weight: inherit;
        color: inherit;
        border: 0px;
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

       
    table.form {
        width: 100%;
    }
    table.form th {
        text-align: right;
        width: auto;
        min-width: 12ch;
    }
    table.form td {
        width: 100%;
    }
    
    div.error {
        color: red;
        font-weight: bold;
    }
    
    .protoPreview {
        overflow: hidden;
    }
    
    </style>
    