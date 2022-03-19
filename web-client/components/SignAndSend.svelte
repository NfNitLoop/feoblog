<!-- 
    Component that knows how to sign and send an item 
    Accepts an Item, presents a sign & send box for the logged-in user.
    dispatches:
     * "sendSuccess", {userID, signature}
-->
<div class="sendBox inputWhiteBox" transition:slide|local>

    
    {#if !validSignature}
        {#if displayErrors.length > 0}
            <div class="errorBox">
            <ul>
                {#each displayErrors as error}
                    <li>{error}</li>
                {/each}
            </ul>
            </div>
        {:else if warnings.length > 0}
            <div class="warningBox">
            <ul>
                {#each warnings as warning}
                    <li>{warning}</li>
                {/each}
            </ul>
            </div>
        {/if}
        <!-- 
            When password managers (at least, Enpass) fill out the password,
            they look in the same <form> for a username to fill in. This
            extraneous <form> and <input> are to keep it from mucking with other
            fields.
        -->
        <form>
            <input type="text" name="login" autocomplete="username" placeholder="here to satisfy password managers">
            <InputBox 
                inputType="password"
                label="Private Key"
                placeholder=""
                bind:value={privateKey}
                bind:errorMessage={privateKeyError}
            />
            <Button on:click={sign} disabled={!privateKey || anyErrors || !validPrivateKey}>Sign</Button>
        </form>
    {:else}
        <InputBox
            label="Signature"
            value={signature}
         />
        <div class="buttons">
            <Button on:click={submit} disabled={inProgress}>Submit</Button>
        </div>
        <TaskTrackerView {tracker}/>
    {/if}
</div>


<script lang="ts">
import { getContext } from "svelte";
import { slide } from "svelte/transition"
import type { Writable } from "svelte/store";
import type { Item, File as PFile } from "../protos/feoblog";
import type { AppState } from "../ts/app";
import { Signature } from "../ts/client";
import Button from "./Button.svelte";
import InputBox from "./InputBox.svelte"
import TaskTrackerView from "./TaskTrackerView.svelte"
import nacl from "tweetnacl";
import { FileInfo, Mutex, TaskTracker } from "../ts/common";
import { decodeBase58, decodeBase58Check, encodeBase58 } from "../ts/fbBase58";

let appState: Writable<AppState> = getContext("appStateStore")
export let item: Item

// Errors sent to us from outside, which can prevent sign & send.
export let errors: string[] = []
// Warnings we should expose to the user, but don't necessarily prevent a send.
export let warnings: string[] = []

export let navigateWhenDone = true
// Called when we've successfully sent the item. 
export let onSendSuccess = () => {}

// Attachments SignAndSend should send w/ the Item:
export let attachments: FileInfo[] = []


$: itemBytes = item.serialize()

$: userID = $appState.requireLoggedInUser()

let privateKey = ""
let signature = ""

// Additional errors we check before sending any Items:
let ourErrors: string[] = []
$: anyErrors = (errors.length > 0) || (ourErrors.length > 0) 
$: ourErrors = function() {
    let errs: string[] = []

    if (!itemBytes || itemBytes.length == 0) {
        errs.push("No Item received to sign.")
    }

    // Note: eventually we'll need to check non-post types here. (Profiles).
    // But the rules may be different.
    let itemFiles: PFile[]|null = item.post?.attachments?.file
    let itemFilesMap = new Map<string, PFile>()
    if (itemFiles) {
        for (let pf of itemFiles) {
            itemFilesMap.set(pf.name, pf)
        }

        if (itemFiles.length != itemFilesMap.size) {
            errs.push("Duplicate file attachment names.")
        }
    }

    // Watch for (some) mismatches between the Item and attachments:
    for (let attachment of attachments) {
        if (!itemFilesMap.has(attachment.name)) {
            errs.push(`Attachment "${attachment.name}" missing from Item.attachment.files.`)
        }
    }
    if (attachments.length != itemFilesMap.size) {
        errs.push(`Found ${attachments.length} attachments, but ${itemFilesMap.size} declared in Item metadata`)
    }
    
    return errs
}()

// Show our own checks last, in case they might be duplicates w/ those provided by the caller:
$: displayErrors = errors.length > 0 ? errors : ourErrors



$: validSignature = function(): boolean {
    if (!userID || !signature || !itemBytes) {
        return false
    }

    let isValid = false
    try {
        let sig = Signature.fromString(signature)
        isValid = sig.isValidSync(userID, itemBytes)
    } catch (error) {
        console.error("Error validating signature:", error)
    }

    // Re-validating a signature on every keypress is *expensive*.
    // If we've started editing and this signature is no longer valid, delete it so
    // that we can short-circuit (above)
    if (!isValid) {
        signature = ""
    }

    return isValid
}()

// Error to display about the private key:
$: privateKeyError = function() {
    if (privateKey.length == 0) {
        return "";
    }
    
    let buf: Uint8Array;
    try {
        buf = decodeBase58(privateKey)
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
        buf = decodeBase58Check(privateKey)
    } catch (e) {
        return "Invalid Password"
    }

    
    let keypair = nacl.sign.keyPair.fromSeed(buf);
    
    let pubKey = encodeBase58(keypair.publicKey)
    if (pubKey != userID.toString()) {
        return "Private key does not match user ID."
    }

    return ""    
}()
$: validPrivateKey = !privateKeyError


// Create a signature, delete the password.
function sign() {
    if (privateKeyError) {
        console.error("Shouldn't be able to call sign w/ invalid private key.")
        return
    }

    if (!itemBytes) throw `No bytes to sign.`

    let buf = decodeBase58Check(privateKey)
    let keypair = nacl.sign.keyPair.fromSeed(buf);
    let binSignature = nacl.sign.detached(itemBytes, keypair.secretKey)
    signature = encodeBase58(binSignature)

    // Delete the privateKey, we don't want to save it any longer than
    // necessary:
    privateKey = ""
}

let inProgress = false
let sendMutex = new Mutex()
sendMutex.lockNotifier = (locked) => {inProgress = locked}
let tracker = new TaskTracker()

async function submit() {
    if (inProgress) {
        console.error("submit() while inProgress!? Shouldn't be possible")
        return
    }
    sendMutex.run(async () => {
        await tracker.run("Sending", doSubmit)
    })
}

async function doSubmit(tracker: TaskTracker): Promise<void> {
    if ( anyErrors || !validSignature) {
        console.error("Submit clicked when not valid");
        return
    }

    // We've already checked this, but assert it for TypeScript:
    if (!itemBytes) {
        console.error("ItemBytes is not set!?")
        return
    }


    let sig = Signature.fromString(signature)

    let client = $appState.client

    await tracker.runSubtask("Sending Item", async (tracker) => {
        let result = await client.putItem(userID, sig, itemBytes!)
        tracker.log(`Success: ${result.status}: ${result.statusText}`)
    })

    let uploads = attachments
    if (uploads.length > 0) {
        await tracker.runSubtask("Uploading attachments", async (tracker) => {
            for (let upload of uploads) {
                await tracker.runSubtask(`Uploading "${upload.name}" (${upload.readableSize})`, async (tracker) => {
                    await client.putAttachment(userID, sig, upload.name, upload.file)
                })
            }
        })
    }

    // Save this before onSendSuccess(), because it could change values:
    let navigateDestination = `#/u/${userID}/i/${sig}/`

    await tracker.runSubtask("executing onSendSuccess()", async (tracker) => {
        // Mostly here to catch and report an exception in the handler:
        onSendSuccess()
    })
    
    if (tracker.errorCount > 0) {
        // Do not navigate.
        return
    }

    if (navigateWhenDone) {
        window.location.hash = navigateDestination
    }
}

</script>

<style>
input[name="login"] {
    /* Used only to help password managers not paste the userID in the wrong place. */
    display: none;
}

.errorBox, .warningBox{
    margin-bottom: 1rem;
    border-radius: 3px; /* Matches inputBox */
    border: solid 1px red;
    background-color: #ff000014;
}



.errorBox::before {
    content: "⚠ Errors:";
    margin: 1rem;
    font-weight: bold;
    display: block;
}

.warningBox {
    border: solid 1px #ffaa00;
    background-color: #ffaa0014;
}

.warningBox::before {
    content: "⚠ Warnings:";
    margin: 1rem;
    font-weight: bold;
    display: block;
}

</style>