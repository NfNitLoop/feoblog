<!--
    Component for editing a profile. Used from within EditorWithPreview
-->
<div class="item editPane">
    <div class="body">
        <h1><input type="text" bind:value={title} placeholder="Title (Optional)"></h1>
        <TimestampEditor
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
import { Attachments, File, Item, Post } from "../protos/feoblog";
import moment from "moment";
import FileAttachments from "./FileAttachments.svelte"
import type {FileInfo} from "../ts/common"
import { getContext, onMount } from "svelte";
import type { AppState } from "../ts/app";
import type { Writable } from "svelte/store";
import type { UserID } from "../ts/client";

export let files: FileInfo[] = []
// The FileAttachments component itself:
let fileAttachments: FileAttachments

let title = ""
let text = ""
let timestampMsUTC = moment().valueOf()
let offsetMinutes = moment().utcOffset()
let timestampString: string // bound to the TimstampEditor.value
let timestampErrors: string[] = []

let textarea: ExpandingTextarea

let appState: Writable<AppState> = getContext("appStateStore")
let userID = $appState.loggedInUser
let localStorage = new LocalStorageProxy()
let initialized = false


// localStorage key to a JSON object that maps userID -> PostData.
const draftsKey = "editPost.drafts"
type Drafts = { [key: string]: PostData }

async function loadDraft() {
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
    // Also empty timestamps are weird for now so skip those.
    if (!draft.text) { return }
    if (!draft.timestampString) { return }

    title = draft.title
    text = draft.text
    timestampString = draft.timestampString        
}
onMount(loadDraft)

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
export let item: Item
$: item = function() {
    let itm = new Item({
        timestamp_ms_utc: timestampMsUTC,
        utc_offset_minutes: offsetMinutes,
        post: new Post({
            title,
            body: text,
        })
    })

    if (files.length > 0) {
        let attachments: File[] = []
        for (let info of files) {
            let file = new File({
                hash: info.hash.bytes,
                size: info.size,
                name: info.name,
            })
            attachments.push(file)
        }

        itm.post.attachments = new Attachments({file: attachments})
    }

    return itm
}()


export let validationErrors: string[] = []
$: validationErrors = function(): string[] {
    let errs = [...timestampErrors]

    if (!(title.trim() || text.trim())) {
        errs.push("Can't post with an empty title and body.")
    }

    return errs
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