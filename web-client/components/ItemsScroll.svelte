<!--
    "infinite" scrolling for a collection of items.

    TODO: (see: HEAD^^) un-shrink images after they've loaded, instead of waiting until a page scrolls into view.
-->
<ShowWhen condition={showTopStatus}>
    <ItemBox>
        {#if noMoreTop}
            <p>You've reached the newest items on this page as of {$elapsedSinceNoMoreTop.elapsedRelative}</p>
            <p><Button disabled={$elapsedSinceNoMoreTop.elapsedMs < 10_000} on:click={checkMoreTop}>Check For New Items</Button></p>
        {:else if topLoader != null}
            <p>Loading...</p>
        {:else}
            <!-- TODO: This seems hacky.  If they can see it, should we not be loading it? -->
            <p>Scroll up to load more.</p>
        {/if}
    </ItemBox>
</ShowWhen>

{#each $items as entry (entry) }
    <ItemView 
        userID={entry.userID.asBase58}
        signature={entry.signature.asBase58}
        item={entry.item}
        shrinkImages={shrinkImages(entry)}
        active={entry.signature.asBase58 == activeItemSignature}
        on:enteredPage={itemEnteredScreen}
        on:leftPage={itemLeftScreen}
        on:mousemove={() => onItemMouseEnter(entry)}
    />
{/each}
<ItemBox><p>
    {#if noMoreBottom}
        No more items to display.
    {:else}
        Loading...
    {/if}
</p></ItemBox>
<svelte:window bind:scrollY on:keyup={onWindowKeyUp} />

<script lang="ts">
import type { PageEvent } from "./ItemView.svelte";
import type { ItemListEntry } from "../protos/feoblog";
import type { DisplayItem, ItemOffsetParams } from "../ts/client";
import { ItemFilter, LazyItemLoader } from "../ts/client";

import { ConsoleLogger, delayMs, InfiniteScroll } from "../ts/common";
import ItemView from "./ItemView.svelte";
import type { Writable } from "svelte/store";
import type { AppState } from "../ts/app";
import { getContext, onDestroy, tick } from "svelte";
import ItemBox from "./ItemBox.svelte";
import ShowWhen from "./widgetes/ShowWhen.svelte";
import Button from "./Button.svelte";
import { ElapsedTime } from "../ts/asyncStore";

const logger = new ConsoleLogger({prefix: "<ItemScroll>"}).withDebug()
logger.debug("Created logger. (fresh load)")

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
let showTopStatus = false
let elapsedSinceNoMoreTop = new ElapsedTime()

$: logger.debug("scrollPos", scrollPos)
$: logger.debug("noMoreBottom", noMoreBottom)
$: logger.debug("noMoreTop", noMoreTop)


// if user changes itemfilter, or manually navigates to new scrollPosition, reload.
$: onFilterChange(itemFilter, scrollPos)
function onFilterChange(..._changedFields: unknown[]) { 
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

    let displayItem: (di: DisplayItem) => Promise<void>
    let endReached: () => void

    let isBottom = typeof(offset.before) == "number"
    logger.debug("Reinit loader for", isBottom ? "bottom":"top")
    if (isBottom) {
        noMoreBottom = false
        endReached = () => { noMoreBottom = true }
        displayItem = async (di) => { await items.pushBottom(di) }
    } else {
        noMoreTop = false
        endReached = () => { noMoreTop = true; elapsedSinceNoMoreTop.startNow() }
        displayItem = async (di) => { await items.pushTop(di) }
    }

    let newLoader = new LazyItemLoader({
        client: $appState.client,
        itemEntries: itemLoader,
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
$: logger.debug("visible", visibleElements.map(e => e.item?.timestamp_ms_utc))

function itemEnteredScreen(event: CustomEvent<PageEvent>) {
    visibleElements = [...visibleElements, event.detail]

    logger.debug("Item entered page", event.detail.signature.substring(0, 10))

    let ts = event.detail.item?.timestamp_ms_utc
    if (ts && (!shrinkWatermark || ts > shrinkWatermark)) {
        shrinkWatermark = ts
        logger.debug("new shrinkWatermark", shrinkWatermark)
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
        logger.debug("Setting initial watermark", ts)
        shrinkWatermark = ts
        return false
    }

    let shrink = ts > shrinkWatermark

    if (shrink) {
        logger.debug(entry.signature.asBase58.substring(0, 10), "shrinkImages", shrink)
    }

    return shrink
}


let previousScrollY = 0
// Can be undefined before first event fires:
let scrollY: number|undefined

$: onVerticalScroll(scrollY)

function onVerticalScroll(newY: number|undefined) {
    if (newY === undefined) { return }

    logger.debug("onVerticalScroll previous", previousScrollY, "new", newY)
    let oldScrollY = previousScrollY
    previousScrollY = newY

    let winHeight = window.innerHeight
    // -5 because docHeight is a float and may not be precisely == in long docs (observed in Chrome)
    let docHeight = document.body.scrollHeight - 5

    // The threshold for being "near" is one screen:
    let nearHeight = winHeight

    // If we've already loaded everything at the bottom, don't care if we're at/near there.
    if (!noMoreBottom) {
        // Note: Check bottom first because it's the preferred scroll direction:
        let bottomY = newY + winHeight
        if (bottomY >= docHeight && bottomY > oldScrollY) {
            logger.debug("bumpedBottom")
            bumpedBottom()
            return
        }

        // Note: don't continue to fire "nearBottom" if there's no more to load there.
        // Need to allow fallthrough to bumped/near top for short documents.
        if (bottomY + nearHeight >= docHeight) {
            logger.debug("nearBottom")
            nearBottom()
            return
        }
    }
   
    // With momentum-based scrolling in Safari, oldScroll might've been < 0!
    // TODO: Bug here when manually changing ?ts= value, oldScrollY is not reset.
    if (oldScrollY != 0 && newY == 0) {
        logger.debug("bumpedTop", "olldScrollY", oldScrollY, "newY", newY)
        bumpedTop()
        return
    }

    if (newY < nearHeight) {
        showTopStatus = true
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
    logger.debug("No more items at bottom, init top loader")
    topLoader = reInitLoader(topLoader, {after: scrollPos})
}

function nearTop() {
    // Only when we bump the top do we try loading in that direction.
    // But we make an exception if we started at the bottom:
    if (!haveBumpedTop) { return }
    logger.debug("nearTop")

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

function checkMoreTop() {
    logger.log("checkMoreTop()")
    topLoader?.stop()
    topLoader = null
    bumpedTop()
}


//----------- Stuff for keyboard navigation --------------- 

let activeItemSignature: string|undefined = undefined

// Note, we don't bind mouseenter (above) because browsers fire that when you scroll an item into view under the mouse cursor. boo.
function onItemMouseEnter(item: DisplayItem) {
    logger.debug("onItemMouseEnter event", item.signature.asBase58)
    activeItemSignature = item.signature.asBase58
}

function onWindowKeyUp(event: KeyboardEvent) {
    let {key, target} = event
    if (!(target instanceof Element)) {
        logger.debug("onWindowKeyUp target was not an Element")
        return
    }
    
    logger.debug("keyUp", key, "on", target.tagName, event)
    if (target.tagName != "BODY") { return }
    if (key == "k") {
        selectItem("prev")
    } else if (key == "j") {
        selectItem("next")
    }
}

function selectItem(direction: "prev"|"next") {
    let allItems = $items
    let index = allItems.findIndex(it => it.signature.asBase58 == activeItemSignature)
    if (index < 0) {
        // No active item? Just select the first one.
        index = 0
    } else if (direction == "prev") { 
        index--
    } else if (window.scrollY == 0) {
        // We're at the top, we just want to scroll to this element:
        index = 0
    } else { 
        index++ 
    }

    if (index < 0) {
        // Just go all the way to the top of the page. 
        // 1. looks nice
        // 2: triggers loading if we haven't started that yet.
        window.scrollTo({top: 0})
        return
    }
    if (index >= allItems.length) {
        logger.debug("selectItem: User tried to navigate too far")
        return
    }

    if (index == 0 && window.scrollY == 0) {
        // we've scrolled to the top, now just scroll to this element:
    }

    let newItem = allItems[index]
    activeItemSignature = newItem.signature.asBase58
    
    let element = document.getElementById(newItem.signature.asBase58)
    if (!element) {
        logger.debug("No element for", newItem.signature.asBase58)
        return
    }

    let top = element.offsetTop
    logger.debug("offsetTop:", top)

    top -= 5
    $appState.scrollMutex.run(async () => {
        window.scrollTo({top})
        // Hold the lock through the scroll:
        logger.debug("before delayMs()")
        // This feels quite hacky.  tick() doesn't work becasue there were no DOM changes to make here.
        await delayMs(50)
        logger.debug("after delayMs()")
    })

}

</script>