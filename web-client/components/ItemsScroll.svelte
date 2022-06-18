<!--
    "infinite" scrolling for a collection of items.

    TODO: (see: HEAD^^) un-shrink images after they've loaded, instead of waiting until a page scrolls into view.
-->
{#each $items as entry (entry) }
    <ItemView 
        userID={entry.userID.toString()}
        signature={entry.signature.toString()}
        item={entry.item}
        shrinkImages={shrinkImages(entry)}
        on:enteredPage={itemEnteredScreen}
        on:leftPage={itemLeftScreen}
    />
    <!-- 
    {#if entry.item?.timestamp_ms_utc == firstVisible?.item?.timestamp_ms_utc}
        <div>👆⬆ This guy is the first visible</div>
    {/if}
    -->
{/each}
<div class="item"><div class="body"><p>
    {#if noMoreBottom}
        No more items to display.
    {:else}
        Loading...
    {/if}
</p></div></div>
<svelte:window bind:scrollY />

<script lang="ts">
import type { PageEvent } from "./ItemView.svelte";
import type { ItemListEntry } from "../protos/feoblog";
import type { DisplayItem, ItemOffsetParams } from "../ts/client";
import { ItemFilter, LazyItemLoader } from "../ts/client";

import { ConsoleLogger, InfiniteScroll } from "../ts/common";
import ItemView from "./ItemView.svelte";
import type { Writable } from "svelte/store";
import type { AppState } from "../ts/app";
import { getContext, onDestroy } from "svelte";

const LOGGER = new ConsoleLogger().withDebug()

export let scrollPos: number
export let createItemLoader: (offset: ItemOffsetParams) => AsyncGenerator<ItemListEntry>|null

export let itemFilter: ItemFilter = ItemFilter.allowAll()

let appState: Writable<AppState> = getContext("appStateStore")

let bottomLoader: LazyItemLoader|null = null 

// We only load top if the user bumps it.
// See bumpedTop()
let topLoader: LazyItemLoader|null = null

// We've reached the end of the items at the bottom/top:
let noMoreBottom = false
let noMoreTop = false


$: onFilterChange(itemFilter)
function onFilterChange(filter: ItemFilter) { 
    topLoader?.stop()
    topLoader = null
    bottomLoader?.stop()
    bottomLoader = null

    items.clear()
    bottomLoader =  reInitLoader(bottomLoader, {before: scrollPos + 1})
}


function reInitLoader(oldLoader: LazyItemLoader|null|undefined, offset: ItemOffsetParams): LazyItemLoader|null {
    // This can happen when Svelte re-uses the component:
    oldLoader?.stop()
    let itemLoader = createItemLoader(offset)
    if (!itemLoader) { return null }

    let isBottom = typeof(offset.before) == "number"
    LOGGER.debug("Reinit logger", isBottom ? "bottom":"top")
    let continueLoading = (() => false)
    let displayItem = isBottom ? (
        async (di: DisplayItem) => { await items.pushBottom(di) }
    ) : (
        async (di: DisplayItem) => { await items.pushTop(di) }
    )
    let endReached = isBottom ? () => { noMoreBottom = true } : () => { noMoreTop = true }

    let newLoader = new LazyItemLoader({
        client: $appState.client,
        itemEntries: itemLoader,
        continueLoading,
        // displayItem: async (di) => {
        //     console.log("display", isBottom? "bottom" : "top", di.item.timestamp_ms_utc)
        //     await displayItem(di)
        // },
        displayItem,
        endReached,
        itemFilter,
    })

    newLoader.displayMoreItems()
    return newLoader
}

let items = new InfiniteScroll<DisplayItem>()

let visibleElements: PageEvent[] = []
// $: console.log("visible", visibleElements.map(e => e.item?.timestamp_ms_utc))

function itemEnteredScreen(event: CustomEvent<PageEvent>) {
    visibleElements = [...visibleElements, event.detail]

    console.debug("Item entered page", event.detail.signature.substring(0, 10))

    let ts = event.detail.item?.timestamp_ms_utc
    if (ts && (!shrinkWatermark || ts > shrinkWatermark)) {
        shrinkWatermark = ts
        console.debug("new shrinkWatermark", shrinkWatermark)
    }
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
    scrollPos = ts
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
    // TODO: I can probably replace most of this by delegating to CancelTimer.
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


// TODO: SHould be (timestamp, signature) for a full ordering. 
// The top timestamp that's ever been visible.
// Things before this have shrunken images to avoid scroll issues.
let shrinkWatermark: number|undefined = undefined

function shrinkImages(entry: DisplayItem): boolean {
    // For the very first item, don't wait for an itemEnteredScreen event
    // If it doesn't happen from scrolling down, scrolling up will not get a shrinkImage=true, which will
    // cause scrolling issues.
    let ts = entry.item.timestamp_ms_utc
    if (!shrinkWatermark) {
        console.log("Setting initial watermark", ts)
        shrinkWatermark = ts
        return false
    }

    let shrink = ts > shrinkWatermark

    if (shrink) {
        console.debug(entry.signature.asBase58.substring(0, 10), "shrinkImages", shrink)
    }

    return shrink
}


let previousScrollY = 0
let scrollY: number

$: onVerticalScroll(scrollY)

function onVerticalScroll(newY: number) {
    let oldScrollY = previousScrollY
    previousScrollY = newY

    let winHeight = window.innerHeight
    // -5 because docHeight is a float and may not be precisely == in long docs (observed in Chrome)
    let docHeight = document.body.scrollHeight - 5

    // The threshold for being "near" is one screen:
    let nearHeight = winHeight

    // Note: Check bottom first because it's the preferred scroll direction:
    let bottomY = newY + winHeight
    if (bottomY >= docHeight && bottomY > oldScrollY) {
        LOGGER.debug("bumpedBottom")
        bumpedBottom()
        return
    }

    if (bottomY + nearHeight >= docHeight) {
        LOGGER.debug("nearBottom")
        nearBottom()
        return
    }

    // With momentum-based scrolling in Safari, oldScroll might've been < 0!  
    if (oldScrollY != 0 && newY == 0) {
        LOGGER.debug("bumpedTop")
        bumpedTop()
        return
    }

    if (newY < nearHeight) {
        // LOGGER.debug("nearTop")
        nearTop()
        return
    }
}

// top/bottom loaders can have 3 states:
// loader == null -- has never been initialized, or was cleared.
// !loader.hasMore -- loader ran out of items.
// loader.hasMore -- ready to load more items.

let haveBumpedTop = false
function bumpedTop() {
    haveBumpedTop = true
    nearTop()
}

$: if (noMoreBottom && !topLoader) {
    // We were never able to bump the top. Start loading it:
    topLoader = reInitLoader(topLoader, {after: scrollPos})
}

function nearTop() {
    // Only when we bump the top do we try loading in that direction.
    // But we make an exception if we started at the bottom:
    if (!haveBumpedTop) { return }
    LOGGER.debug("nearTop")

    if (!topLoader) {
        let topTs = $items[0]?.item?.timestamp_ms_utc
        if (!topTs) { return }
        topLoader = reInitLoader(topLoader, {after: topTs})
        if (shrinkWatermark) {
            // TODO: WHy did I think I needed to do this?
            // shrinkWatermark = undefined
        }
    }

    if (!topLoader?.hasMore) { return }

    // Reset bottomLoader because we may end up truncating the list.
    bottomLoader?.stop()
    bottomLoader = null
    topLoader.displayMoreItems()
}

function bumpedBottom() {
   // No real need to distinguish this case:
   nearBottom()
}

function nearBottom() {
    
    if (!bottomLoader) {
        let myItems = $items
        let end = myItems.length - 1
        let bottomTs = myItems[end]?.item?.timestamp_ms_utc
        if (!bottomTs) { return }
        bottomLoader = reInitLoader(bottomLoader, {before: bottomTs})
    }
    
    if (!bottomLoader?.hasMore) { return }

    topLoader?.stop()
    topLoader = null
    // LOGGER.debug("loadMore (bottom)")
    bottomLoader.displayMoreItems()
}
</script>