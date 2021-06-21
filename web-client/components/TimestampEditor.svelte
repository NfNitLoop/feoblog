<!--
    Select a timetamp!
    Right now, is a text-based timestamp editor. 
    TODO: In the future, add a button to choose a date/time more easily.    
-->
<div class="timestamp timestampEditor" class:errorBox>
    <input type="text" on:blur={onBlur} bind:value/>
</div>

<script lang="ts" context="module">
// Strictly parse one of these non-ambiguous timestamps. (MUST include time zone.)
const DATE_FORMATS = [
    // Preferred:
    "YYYY-MM-DD HH:mm:ss.SSS ZZ",
    // May drop milliseconds:
    "YYYY-MM-DD HH:mm:ss ZZ",
    // ... and seconds:
    "YYYY-MM-DD HH:mm ZZ",
]

</script>

<script lang="ts">
import moment from "moment";

// inout
export let value = ""

// out: parsed timestamp values, or errors:
export let msUTC: number
export let offsetMinutes: number
export let errors: string[] = []


if (msUTC) {
    setStringFromMs()
} else {
    setNow()
}

// only update from value -> (msUTC,offsetMinutes,errors)
$: updateFromValue(value)
function updateFromValue(value: string) {
    errors = []

    let date = parseDate(value)
    if (!date.isValid()) {
        errors = ["Invalid date"]
        return
    }

    msUTC = date.valueOf()
    offsetMinutes = date.utcOffset()
}

function parseDate(str: string): moment.Moment {
    let date: moment.Moment|undefined = undefined
    if (DATE_FORMATS.length == 0) {
        throw "DATE_FORMATS is empty"
    }
    for (let i in DATE_FORMATS) {
        // keep the parsed offset in the Moment so we can render/save it.
        date = moment.parseZone(str, DATE_FORMATS[i], true)
        if (date.isValid()) {
            return date
        }
    }
    return date as moment.Moment
}

function setNow() {
    value = moment().format(DATE_FORMATS[0])
}

function setStringFromMs() {
    value = moment(msUTC).utcOffset(offsetMinutes).format(DATE_FORMATS[0])
}

// If the user broke the timestamp, return it to its correct format:
function onBlur() {
    let parsed = parseDate(value)
    if (parsed.isValid()) { return }

    // A hacky way to set the time to now:
    if (value.trim() == "") {
        setNow()
        return
    }

    // We treat 0 as an undefined time in FeoBlog, since Protobuf can't distinguish it:
    if (msUTC != 0) {
        setStringFromMs()
        return
    }

    // else:
    setNow()
}


$: errorBox = errors.length > 0

</script>

<style>
.errorBox {
    border: 1px solid red;
}
.timestamp {
    font-family: Consolas, monospace;
}
</style>