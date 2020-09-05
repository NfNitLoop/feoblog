<div id="app">
    <div class="postMetadata">
        <table class="form">
            <tr>
                <th><label for="title">Title</label>:</th>
                <td><input type="text" name="title" bind:value={title}></td>
            </tr>
            <tr>
                <th><label for="time">Time</label>:</th>
                <td>
                    <input type="text" name="time" bind:value={timeInput}>
                    {#if timeInputError}
                    <div class="error">{timeInputError}</div>
                    {/if}
                </td>
            </tr>
        </table>
    </div>

    <div class="postPreviewHeader">
        Markdown Preview
    </div>

    <div class="postBody">
        <textarea bind:this={textbox} bind:value={post}></textarea>
        <table class="form">
            <tr>
                <th><label for="userID">User ID</label>:</th>
                <td><input type="text" name="userID" bind:value={userID}></td>
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
                <th><label for="signature">Signature</label>:</th>
                <td><input type="text" name="signature" bind:value={signature} disabled></td>
            </tr>
            <tr>
                <th></th>
                <td>
                    <button name="submit" disabled={!valid}>Submit</button>
                </td>
            </tr>
        </table>
    </div>

    <div id="output">
        <div class="postPreviewHead">
            {#if title}
            <h1 class="title">{ title }</h1>
            {/if}
            <div class="timestamp">{ formattedDate }</div>
        </div>
        
        <div id="markdown-out">{@html markdownOut}</div>
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

</div>


<script lang="ts">
import { onMount } from 'svelte';
import bs58 from "bs58"
import commonmark from "commonmark"
import moment from "moment"
import { Item, Post } from "../protos/feoblog"
import * as nacl from "tweetnacl-ts"

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
let post = "Hello world!"
let textbox; // .value holds `post`
onMount(() => {
    textbox.focus();
    textbox.selectionStart = 0;
    textbox.selectionEnd = textbox.value.length;
});
// focusTextBox()


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
    console.debug(`Got pkey of length ${privateKey.length}`)
    if (privateKey.length == 0) {
        return "";
    }
    
    let buf: Uint8Array;
    try {
        buf = bs58.decode(privateKey)
    } catch (error) {
        return "Not valid base58"
    }

    if (buf.length != 32) {
        return "Password too short"
    }
    
    // TODO: Consider using base58Check so that we can verify valid
    // pkeys if someone tries to type one in.

    let keypair = nacl.sign_keyPair_fromSeed(buf);
    userID = bs58.encode(keypair.publicKey)
    let binSignature = nacl.sign_detached(itemProtoBytes, keypair.secretKey)
    signature = bs58.encode(binSignature)

    // Delete the privateKey, we don't want to save it any longer than
    // necessary:
    privateKey = ""

    console.log("generated signature", signature)
    return ""    
}()

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
    let item = new Item({
        timestamp_ms_utc: timestampMoment.valueOf(),
        utc_offset_minutes: timestampMoment.utcOffset(),
        post: new Post()
    })

    // See: https://github.com/thesayyn/protoc-gen-ts/issues/16
    let postPb = item.post;
    if (title) { postPb.title = title }
    if (post) { postPb.body = post }

    return item;
}()

$: itemProtoBytes = itemProto.serialize()
$: protoSize = itemProtoBytes.length
$: protoHex = bufferToHex(itemProtoBytes)

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
$: valid = errors.length == 0;

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

</script>



<style type="text/css">
    #app {
        display: grid;
        width: 100%;
        grid-template-columns: 1fr 1fr;
        /* max-height: 80vh; */
    }
    
    #app > div {
        margin-left: 0.5em;
        margin-right: 0.5em;
    }
    
    #app .postBody textarea {
        min-height: 20em;
        width: 100%;
    }
    
    #output {
        border: 1px dashed grey;
        border-radius: 1em;
        padding-left: 1em;
        padding-right: 1em;
        overflow: auto;
        min-height: 1em; /* fixes overflow? */
        max-height: 100%;
    
    }
    
    #output .timestamp {
        color: grey;
        font-size: 0.8em;
    }
    
    .postPreviewHead .title {
        margin-bottom: 0px;
    }
    
    table.form {
        width: 100%;
    }
    table.form th {
        text-align: right;
        width: 20%;
        min-width: 12ch;
    }
    table.form td {
        width: 80%;
    }
    
    table.form input[type=text], table.form input[type=password] {
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
    