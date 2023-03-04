<!--
    Component for editing a profile. Used from within EditorWithPreview
-->
<div class="item editPane">
    <div class="body">
        <h1><input type="text" bind:value={title} placeholder="Title (Optional)"></h1>
        <TimestampEditor
            bind:this={timestampEditor}
            bind:value={timestampString}
            bind:msUTC={timestampMsUTC}
            bind:offsetMinutes
            bind:errors={timestampErrors}
        />
        <ExpandingTextarea
            bind:value={text}
            bind:this={textarea}
            on:paste={onPaste}
            placeholder="Your post here 😊"
        />
        <FileAttachments bind:files bind:this={fileAttachments} on:fileAdded={fileAdded}/>
    </div>
</div>

<script lang="ts" context="module">
// Proxies the window.localStorage object. Will warn once if localStorage is not present and then fall back to no-ops.
class LocalStorageProxy {

    private localStorage: Storage|null|undefined = window.localStorage

    constructor() {
        if (!this.localStorage) { 
            console.warn("No local storage. Can't save ")
        }
    }

    getItem(key: string, fallback: string): string {
        if (!this.localStorage) { return fallback }

        let value = this.localStorage.getItem(key)
        return value === null ? fallback : value
    }

    setItem(key: string, value: string) {
        if (!this.localStorage) { return }

        this.localStorage.setItem(key, value)
    }
}

type PostData = {
    title: string
    text: string
    timestampString: string
}

</script>


<script lang="ts">
import ExpandingTextarea from "./ExpandingTextarea.svelte"
import TimestampEditor from "./TimestampEditor.svelte"
import FileAttachments from "./FileAttachments.svelte"
import {FileInfo, getMarkdownInfo} from "../ts/common"
import { getContext, onMount } from "svelte";
import type { AppState } from "../ts/app";
import type { Writable } from "svelte/store";
import type { UserID } from "../ts/client";
import {protobuf as pb} from "../ts/client";
import { DateTime } from "luxon";

export let files: FileInfo[] = []
// The FileAttachments component itself:
let fileAttachments: FileAttachments

const attachmentsPrefix = "files/" // TODO: relocate?
$: attachedFileNames = new Set<string>(files.map((f) => { return f.name }))

let title = ""
let text = ""

let timestampMsUTC = DateTime.local().valueOf()
let offsetMinutes = DateTime.local().offset
let timestampString: string // bound to the TimstampEditor.value
let timestampErrors: string[] = []
let timestampEditor: TimestampEditor
$: { text; title; timestampEditor?.bumpNow() }

let textarea: ExpandingTextarea

let appState: Writable<AppState> = getContext("appStateStore")
let userID = $appState.loggedInUser
let localStorage = new LocalStorageProxy()
let initialized = false

// clear the editPost form. 
// Important so that we don't try to re-load this in-progress post the next time we write a new post.
export function clear() {
    // Clearing these is enough so that the draft won't be loaded:
    title = ""
    text = ""
}

// localStorage key to a JSON object that maps userID -> PostData.
const draftsKey = "editPost.drafts"
type Drafts = { [key: string]: PostData }

function loadDraft() {
    initialized = true

    if (!userID) {
        console.warn("loadDraft() -- no userID, can't load")
        return
    }

    let drafts: Drafts = JSON.parse(localStorage.getItem(draftsKey, "{}"))

    // TODO: Why isn't this |undefined by default?
    let draft: PostData|undefined = drafts[userID.toString()]
    if (!draft) {
        return // no draft to load
    }
    
    // If there's no text, consider there not to be a previous draft.
    if (!draft.text) { return }

    title = draft.title
    text = draft.text
    timestampString = draft.timestampString        
}
onMount(async () => {
    loadDraft()
    textarea.focusEnd()
})

// When any of these fields change, save the draft:
$: saveDraft(userID, {title, text, timestampString})
function saveDraft(userID: UserID|null, postData: PostData) {
    if (!userID) { return }

    // flow:
    // 1. init default state (let ... initializers)
    // 2. Load any previous drafts (onMount(loadDraft))
    // 3. set initialized=true
    // We don't want to try to save our default state from #1 until after we've done #2 (& #3)
    if (!initialized) { return }

    let drafts: Drafts  = JSON.parse(localStorage.getItem(draftsKey, "{}"))
    drafts[userID.toString()] = postData
    localStorage.setItem(draftsKey, JSON.stringify(drafts))
    
}


// Exported so that EditorWithPreview can preview, serialize, & send it for us.
export let item: pb.Item
$: item = function() {
    let post = new pb.Post({
        title,
        body: text,
    })
    let itm = new pb.Item({
        timestampMsUtc: BigInt(timestampMsUTC),
        utcOffsetMinutes: offsetMinutes,
        itemType: { case: "post", value: post }
    })

    if (files.length > 0) {
        let attachments: pb.File[] = []
        for (let info of files) {
            let file = new pb.File({
                hash: info.hash.bytes,
                size: BigInt(info.size),
                name: info.name,
            })
            attachments.push(file)
        }

        post.attachments = new pb.Attachments({file: attachments})
    }

    return itm
}()


export let validationErrors: string[] = []
$: validationErrors = function(): string[] {
    let errs = [...timestampErrors]

    if (!(title.trim() || text.trim())) {
        errs.push("Title and body are empty.")
    }

    return errs
}()

// Check for common markdown mistakes.
// In particular, check for missing attachments, since we don't save attachments
// along with post drafts.
// One missing check:
// if you reference, e.g.:
// [foo]: https://foo.example.com/
// ... but never use it, we don't have a simple way of detecting that.
export let warnings: string[] = []
$: warnings = function(): string[] {
    let warnings: string[] = []

    // Disabling this. It seems like it might be common to post without a
    // title, and I don't want the warning box to become a thing that people come to ignore.
    // if (title.trim() == "") {
    //     warnings.push("Empty title")
    // }

    if (text.trim() == "") {
        warnings.push("Empty body")
    }

    let info = getMarkdownInfo(text)
    for (let ref of info.unlinkedRefs) {
        warnings.push(`Missing link for: ${ref}`)
    }

    let referencedAttachments = new Set(
        [...info.imageDestinations, ...info.linkDestinations]
        .filter((x) => x.startsWith(attachmentsPrefix))
        .map((x) => x.substr(attachmentsPrefix.length))
    )

    for (let attachment of referencedAttachments) {
        if (!attachedFileNames.has(attachment)) {
            warnings.push(`Missing attachment: "${attachment}"`)
        }
    }

    for (let attachment of attachedFileNames) {
        if (!referencedAttachments.has(attachment)) {
            warnings.push(`Unreferenced attachment: "${attachment}"`)
        }
    }

    return warnings
}()

// A file was added via FileAttachments drag & drop.
function fileAdded(event: CustomEvent<FileInfo>) {
    let fi = event.detail

    textarea.addLink({
        text: fi.name,
        href: `files/${fi.name}`,
        asImage: fi.isImage
    })
}


async function onPaste(event: CustomEvent<ClipboardEvent>) {
    let data = event.detail.clipboardData
    if (!data) return;

    for (let i = 0; i < data.files.length; i++) {
        let file = data.files[i]
        if (file.type == "image/png") {
            fileAttachments.addFile(file)            
        } else {
            console.warn("Pasted unknown file type:", file.type)
        }
    }
}

</script>

<style>
h1 {
    margin-bottom: 0px;
}
</style>