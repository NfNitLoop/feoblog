<!--
    "infinite" scrolling for a collection of items.
-->

{#each $items as entry (entry) }
    <ItemView 
        userID={entry.userID.toString()}
        signature={entry.signature.toString()}
        item={entry.item}
        on:enteredPage={itemEnteredScreen}
        on:leftPage={itemLeftScreen}
    />
    <!-- 
    {#if entry.item?.timestamp_ms_utc == firstVisible?.item?.timestamp_ms_utc}
        <div>👆⬆ This guy is the first visible</div>
    {/if}
    -->
{/each}
<div class="item"><div class="body">
    {#if !noMoreBottom}
        No more items to display.
    {:else}
        Loading...
    {/if}
</div></div>
<VisibilityCheck on:itemVisible={onEndIsVisible} bind:visible={endIsVisible}/>


<script lang="ts">
import type { PageEvent } from "./ItemView.svelte";
import type { ItemListEntry } from "../protos/feoblog";
import type { DisplayItem, ItemOffsetParams } from "../ts/client";
import { ItemFilter, LazyItemLoader } from "../ts/client";

import { InfiniteScroll, observable } from "../ts/common";
import ItemView from "./ItemView.svelte";
import VisibilityCheck from "./VisibilityCheck.svelte";
import type { Writable } from "svelte/store";
import type { AppState } from "../ts/app";
import { getContext, onDestroy } from "svelte";



export let scrollPos: number
export let createItemLoader: (offset: ItemOffsetParams) => AsyncGenerator<ItemListEntry>|null

export let itemFilter: ItemFilter = ItemFilter.allowAll()

let appState: Writable<AppState> = getContext("appStateStore")

let bottomLoader: LazyItemLoader|null
$: bottomLoader = reInitLoader(bottomLoader, {before: scrollPos + 1})
// TODO
// $: topLoader = reInitLoader(topLoader, {after: scrollPos})

// We've reached the end of the items at the bottom/top:
let noMoreBottom = false
let noMoreTop = false

function reInitLoader(oldLoader: LazyItemLoader|null|undefined, offset: ItemOffsetParams): LazyItemLoader|null {
    // This can happen when Svelte re-uses the component:
    if (oldLoader) {
        oldLoader.stop()
    }
    let itemLoader = createItemLoader(offset)
    if (!itemLoader) { return null }

    let newLoader = new LazyItemLoader({
        client: $appState.client,
        itemEntries: itemLoader,
        // TODO:
        continueLoading: () => endIsVisible,
        displayItem: async (di) => {
            await items.pushBottom(di)
        },
        endReached: () => { noMoreBottom = true },
        itemFilter,
    })

    newLoader.displayMoreItems()
    return newLoader
}

let items = new InfiniteScroll<DisplayItem>({getScrollElement: () => firstVisible?.element || null})

let visibleElements: PageEvent[] = []
// $: console.log("visible", visibleElements.map(e => e.item?.timestamp_ms_utc))

function itemEnteredScreen(event: CustomEvent<PageEvent>) {
    visibleElements = [...visibleElements, event.detail]
}

function itemLeftScreen(event: CustomEvent<PageEvent>) {
    // Could be faster but this should be a  super small array anyway.
    visibleElements = visibleElements.filter(e => e.signature.toString() != event.detail.signature)
}


$: firstVisible = getFirstVisible(visibleElements)
function getFirstVisible(events: PageEvent[]): PageEvent|null {
    if (events.length == 0) { return null }

    let event = null
    for (const e of events) {
        if (!e.item) { continue }
        let ts = e.item.timestamp_ms_utc
        if (event == null || ts > event.item!.timestamp_ms_utc) {
            event = e
        }
    }

    return event
}

$: saveScrollPosition(firstVisible)
function saveScrollPosition(event: PageEvent|null) {
    if (!event) { return }
    let ts = event.item?.timestamp_ms_utc
    if (!ts) { return }

    historyThrottle.setParam("ts", `${ts}`)
}


let endIsVisible = false
function onEndIsVisible() {
    bottomLoader?.displayMoreItems()
}

/**
 * Provide throttled access to the window.history API via timers.
 * 
 * Both Chrome and Firefox behave badly if you call the history API too frequently,
 * which can happen when a user is quickly scrolling through a page.
 * Chrome will display a warning about throttling the API, and then the URL bar
 * just doesn't update for a long while.
 * Firefox will actually throw an exception, which can be bad for Svelte's continued
 * operation.
 */
 class HistoryThrottle {
    minDelayMS = 500

    #lastReplaceMs = 0

    // The last timer we started.
    #timer: number|undefined = undefined

    replaceData = {}
    replaceTitle = "FeoBlog Scroll State"
    replaceURL: string|undefined = undefined

    setParam(name: string, value: string) {
        let spaLoc = window.location.hash.substr(1)
        let parts = spaLoc.split("?")
        let params = new URLSearchParams(parts[1] ?? "")
        params.set(name, value)

        let newURL = new URL(window.location.href)
        newURL.hash = `${parts[0]}?${params}`
        this.scheduleReplaceState(newURL.toString())
    }

    scheduleReplaceState(url: string) {
        this.replaceURL = url

        let now = new Date().valueOf()
        let delta = now - this.#lastReplaceMs
        let timeToNext = this.minDelayMS - delta
        if (this.#timer) {
            // a replace is already scheduled. Just wait for that.
            return
        }
        if (timeToNext <= 0) {
            // No need to schedule, just do it now:
            this.replaceState()
            return
        }

        this.#timer = setTimeout(() => this.replaceState(), timeToNext)
    }

    private replaceState() {
        this.#lastReplaceMs = new Date().valueOf()
        this.#timer = undefined
        window.history.scrollRestoration = 'manual'
        window.history.replaceState(this.replaceData, this.replaceTitle, this.replaceURL)
    }

    cancel() {
        clearTimeout(this.#timer)
        this.#timer = undefined
    }
}
let historyThrottle = new HistoryThrottle()
onDestroy(() => {
    historyThrottle.cancel()
})


</script>