<div class="timestamp timestampEditor" class:errorBox>
    <input type="text" bind:value/>
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
    
export let msUTC: number
export let offsetMinutes: number
export let errors: string[] = []

let value = ""

if (msUTC) {
    // set Value from time.
    setNow()
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