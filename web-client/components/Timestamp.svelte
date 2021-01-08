<script lang="ts" context="module">
// Shows a timestamp for a post/item.
//
// Depends on /style.css for styling.
//
// TODO:
//  * Show a relative time on mouseover?
//  * User configurable time preferences?
//  * 

import {DateTime, FixedOffsetZone} from "luxon"
    
</script>

<script lang="ts">
export let utc_ms: number = undefined
export let minute_offset: number = undefined
export let href: string = undefined
export let defaultFormat = "yyyy-MM-dd HH:mm:ss ZZZ"

$: dateTime = function() {
    if (!utc_ms) return null

    let dateTime = DateTime.fromMillis(utc_ms)
    let zone = FixedOffsetZone.instance(minute_offset || 0)
    return dateTime.setZone(zone)
}()

$: formattedDate = function() {
    if (!dateTime) return "undefined"

    // TODO: Support format preferences and/or mouseovers.
    return dateTime.toFormat(defaultFormat)
}()

</script>

<div class="timestamp">
{#if href}
    <a {href}>{formattedDate}</a>
{:else}
    {formattedDate}
{/if}
</div>