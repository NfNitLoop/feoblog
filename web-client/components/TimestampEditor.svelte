<!--
    Select a timetamp!
    Right now, is a text-based timestamp editor. 
    TODO: In the future, add a button to choose a date/time more easily.    
-->
<div class="timestamp timestampEditor" class:errorBox>
    <input type="text" bind:value placeholder="Timestamp (default: now)"/>
</div>

<script lang="ts" context="module">
// Strictly parse one of these non-ambiguous timestamps. (MUST include time zone.)
const DATE_FORMATS = [
    // Preferred:
    "yyyy-MM-dd HH:mm:ss.SSS ZZZ",

    // May drop milliseconds:
    "yyyy-MM-dd HH:mm:ss ZZZ",

    // ... and seconds:
    "yyyy-MM-dd HH:mm ZZZ",
]

</script>

<script lang="ts">
import { DateTime, FixedOffsetZone } from "luxon";

// inout
export let value = ""

// out: parsed timestamp values, or errors:
// Note: values may be incorrect if there are errors.
export let msUTC = 1
export let offsetMinutes = 0
export let errors: string[] = []


$: dateTime = parseDate(value)
$: errors = dateTime.invalidReason ? [dateTime.invalidReason] : []

$: {
    if (dateTime.isValid) {
        msUTC = dateTime.valueOf()
        offsetMinutes = dateTime.offset
    }
}

/**
 * The default value when the input box is empty is now.
 * Call this function to bump the value of "now".
 * (It's not incremented automatically so as not to be too slow.)
 */
export function bumpNow() {
    dateTime = parseDate(value)
}



// Parse a date. May return an invalid date if parsing failed.
function parseDate(str: string): DateTime {
    if (DATE_FORMATS.length == 0) {
        throw "DATE_FORMATS is empty"
    }

    if (str === "") {
        // Default = now:
        return DateTime.local()
    }

    for (let i in DATE_FORMATS) {
        // keep the parsed offset in the Moment so we can render/save it.
        let date = DateTime.fromFormat(str, DATE_FORMATS[i], {setZone: true})
        if (date.isValid) {
            return date
        }
    }
    return DateTime.invalid(
        "Could not parse a valid date. Valid formats are: "
        + DATE_FORMATS.map(it => `"${it}"`).join(" or ")
    )
}

function setNow() {
    value = DateTime.local().toFormat(DATE_FORMATS[0])
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