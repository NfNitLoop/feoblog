<!--
    Shows a timestamp for a post/item.

    Depends on /style.css for styling.

    TODO:
     * Show a relative time on mouseover?
     * User configurable time preferences?
     * 
-->

{#if href}
    <a class="timestamp" {href} title={bothDates}>{relDate}</a>
{:else}
    <span class="timestamp" class:absolute={!showRelative} title={bothDates}>{relDate}</span>
{/if}


<script lang="ts">
import {DateTime, FixedOffsetZone} from "luxon"

export let utc_ms: number|undefined = undefined
export let minute_offset: number|undefined = undefined
export let href = ""
export let defaultFormat = "yyyy-MM-dd HH:mm:ss ZZZ"
export let showRelative = true

$: dateTime = function() {
    if (!utc_ms) return null

    let dateTime = DateTime.fromMillis(utc_ms)
    let zone = FixedOffsetZone.instance(minute_offset || 0)
    return dateTime.setZone(zone)
}()

$: fullDate = function() {
    if (!dateTime) return "undefined"

    // TODO: Support format preferences and/or mouseovers.
    return dateTime.toFormat(defaultFormat)
}()

$: bothDates = function() {
    let localNow = DateTime.local()
    if (!dateTime || dateTime.offset == localNow.offset) {
        return fullDate
    }

    let zone = FixedOffsetZone.instance(localNow.offset)
    let localTime = dateTime.setZone(zone)
    let fmtLT = localTime.toFormat(defaultFormat)


    return `${fullDate}\n${fmtLT}`
}()

$: relDate = getRelativeDate(dateTime)

// TODO: I'd like a little bit more detail if something was <7 days ago. 
// Ex:  instead of "3 days ago" maybe  "3 days 7 hours ago"
// See: https://github.com/moment/luxon/issues/1129
function getRelativeDate(dateTime: DateTime|null): string {
    if (!dateTime) return "undefined"

    if (showRelative) {
        return dateTime.toRelative() || "(unknown)"
    }

    return fullDate
}

</script>

<style>
.absolute {
    font-family: Consolas, monospace;
}
</style>
