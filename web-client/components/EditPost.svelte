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
        <ExpandingTextarea bind:value={text} placeholder="Your post here 😊"/>
    </div>
</div>


<script lang="ts">
import ExpandingTextarea from "./ExpandingTextarea.svelte"
import TimestampEditor from "./TimestampEditor.svelte"
import { Item, Post } from "../protos/feoblog";
import moment from "moment";


let title = ""
let text = ""
let timestampMsUTC = moment().valueOf()
let offsetMinutes = moment().utcOffset()
let timestampErrors: string[] = []

// Exported so that EditorWithPreview can preview, serialize, & send it for us.
export let item: Item
$: item = function() {
    return new Item({
        timestamp_ms_utc: timestampMsUTC,
        utc_offset_minutes: offsetMinutes,
        post: new Post({
            title,
            body: text,
        })
    })
}()


export let validationErrors: string[] = []
$: validationErrors = function(): string[] {
    let errs = [...timestampErrors]

    if (!(title.trim() || text.trim())) {
        errs.push("Can't post with an empty title and body.")
    }

    return errs
}()


</script>

<style>
h1 {
    margin-bottom: 0px;
}
</style>