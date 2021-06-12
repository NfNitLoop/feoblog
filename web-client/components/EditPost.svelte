<!--
    Component for editing a profile. Used from within EditorWithPreview
-->
<div class="item editPane">
    <div class="body">
        <h1><input type="text" bind:value={title} placeholder="Title (Optional)"></h1>
        <TimestampEditor
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


<script lang="ts">
import ExpandingTextarea from "./ExpandingTextarea.svelte"
import TimestampEditor from "./TimestampEditor.svelte"
import { Attachments, File, Item, Post } from "../protos/feoblog";
import moment from "moment";
import FileAttachments from "./FileAttachments.svelte"
import type {FileInfo} from "../ts/common"

export let files: FileInfo[] = []
// The FileAttachments component itself:
let fileAttachments: FileAttachments

let title = ""
let text = ""
let timestampMsUTC = moment().valueOf()
let offsetMinutes = moment().utcOffset()
let timestampErrors: string[] = []

let textarea: ExpandingTextarea

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