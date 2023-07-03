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
            {#if security.hasUnencryptedKey}
                <!-- Display nothing about security. User has opted out. -->
            {:else if keyIsUnlocked}
                <div class="unimportant">Key relocks {$unlockedKey.endRelative}</div>
            {:else if security.hasEncryptedKey}
                <InputBox label="Password" inputType="password" bind:value={privateKeyString}/>
            {:else}
                <SecretKeyInput userID={$appState.loggedInUser} bind:valid={validPrivateKey} bind:value={privateKeyString} />
            {/if}
            <Button on:click={sign} disabled={!signEnabled}>Sign</Button>
        </form>
    {:else}
        <InputBox
            label="Signature"
            value={signature}
            disabled={true}
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
import type { AppState } from "../ts/app";
import { PrivateKey, Signature, protobuf as pb, getInner } from "../ts/client";
import Button from "./Button.svelte";
import InputBox from "./InputBox.svelte"
import TaskTrackerView from "./TaskTrackerView.svelte"
import { ConsoleLogger, FileInfo, Mutex, TaskTracker } from "../ts/common";
import { decodeBase58, decodeBase58Check, encodeBase58 } from "../ts/fbBase58";
import SecretKeyInput from "./SecretKeyInput.svelte";
import { SecurityManager } from "../ts/storage";
import { CountDown } from "../ts/asyncStore";

let appState: Writable<AppState> = getContext("appStateStore")
export let item: pb.Item

// Errors sent to us from outside, which can prevent sign & send.
export let errors: string[] = []
// Warnings we should expose to the user, but don't necessarily prevent a send.
export let warnings: string[] = []

export let navigateWhenDone = true
// Called when we've successfully sent the item. 
export let onSendSuccess = () => {}

// Attachments SignAndSend should send w/ the Item:
export let attachments: FileInfo[] = []

let logger = new ConsoleLogger({prefix: "<SignAndSend>"})
logger.debug("loaded")

$: itemBytes = item.toBinary()

$: userID = $appState.requireLoggedInUser()

let validPrivateKey = false

// I'm lazy, used for private string and password:
let privateKeyString = ""

let signature = ""

// Additional errors we check before sending any Items:
let incorrectPassword = false
let ourErrors: string[] = []
$: ourErrors = function() {
    let errs: string[] = []

    if (incorrectPassword) {
        errs.push("Incorrect Password")
    }

    if (!itemBytes || itemBytes.length == 0) {
        errs.push("No Item received to sign.")
    }

    // Note: eventually we'll need to check non-post types here. (Profiles).
    // But the rules may be different.
    let itemFiles: pb.File[]|undefined = getInner(item, "post")?.attachments?.file
    let itemFilesMap = new Map<string, pb.File>()
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
$: anyErrors = displayErrors.length > 0



$: validSignature = function(): boolean {
    if (!userID || !signature || !itemBytes) {
        return false
    }

    let isValid = false
    try {
        let sig = Signature.fromString(signature)
        isValid = sig.isValidSync(userID, itemBytes)
    } catch (error) {
        logger.error("Error validating signature:", error)
    }

    // Re-validating a signature on every keypress is *expensive*.
    // If we've started editing and this signature is no longer valid, delete it so
    // that we can short-circuit (above)
    if (!isValid) {
        signature = ""
    }

    return isValid
}()

$: securityManager = new SecurityManager(appState, $appState)
$: security = securityManager.getSettings(userID.asBase58)
$: unlockedKey = new CountDown(security.unlockedKeyTimeout)
$: keyIsUnlocked = $unlockedKey.remainingMs > 1000
$: signEnabled = (
    security.hasUnencryptedKey 
    || keyIsUnlocked
    || security.hasEncryptedKey
    || validPrivateKey
)

$: logger.debug("validSignature", validSignature)
$: logger.debug("signature:", signature)

// Create a signature, delete the password.
function sign() {
    if (!itemBytes) throw `No bytes to sign.`

    let privateKey: PrivateKey
    if (keyIsUnlocked || security.hasUnencryptedKey) {
        let key = securityManager.getKey(userID.asBase58)
        if (!key) {
            // Uh-oh, maybe the token timed out. Refresh state:
            console.warn("Refreshing view. Couldn't find unlocked key!?")
            appState.update(it => it)
            return
        }
        privateKey = key
    } else if (security.hasEncryptedKey) {
        let key = securityManager.decryptKey(userID.asBase58, privateKeyString)
        if (!key) {
            incorrectPassword = true
            privateKeyString = ""
            return
        }
        incorrectPassword = false
        privateKey = key

    } else {
        privateKey = PrivateKey.fromBase58(privateKeyString)
    }
    securityManager.maybeSaveKey(userID.asBase58, privateKey)

    let binSignature = privateKey.signDetached(itemBytes)
    signature = encodeBase58(binSignature)

    // Delete the privateKey, we don't want to save (here) it any longer than
    // necessary:
    privateKeyString = ""
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
    await sendMutex.run(async () => {
        await tracker.run("Sending", trackerSubmit)
    })

    if (tracker.errorCount == 0 && navigateWhenDone) {
        tracker.clear()
    }

}

async function trackerSubmit(tracker: TaskTracker): Promise<void> {
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
                    await client.putAttachment(userID, sig, upload.name, upload.blob)
                })
            }
        })
    }

    // Save this before onSendSuccess(), because it could change values:
    const isProfileUpdate = item.itemType.case == "profile"
    let navigateDestination = 
        isProfileUpdate ? `#/u/${userID}/profile`
        : `#/u/${userID}/i/${sig}/`

    if (isProfileUpdate) {
        $appState.userProfileChanged()
    }

    await tracker.runSubtask("executing onSendSuccess()", async (tracker) => {
        // Mostly here to catch and report an exception in the handler:
        onSendSuccess()
    })
    
    if (tracker.errorCount > 0) {
        // Do not navigate.
        return
    }

    if (navigateWhenDone) {
        if (window.location.hash == navigateDestination) {
            window.location.reload()
        } else {
            window.location.hash = navigateDestination
        }
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

form {
    margin: 0;
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

.unimportant {
    font-size: 0.8em;
    color: #888;
}

</style>