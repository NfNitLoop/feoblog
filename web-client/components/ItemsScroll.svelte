<!--
    "infinite" scrolling for a collection of items.

    TODO: (see: HEAD^^) un-shrink images after they've loaded, instead of waiting until a page scrolls into view.
-->
{#if showSlot}
    <slot name="whenEmpty"></slot>
{:else}
    <ShowWhen condition={showTopStatus}>
        <ItemBox>
            {#if loadingTop}
                <p>Loading...</p>
            {:else}
                {#if !loadTopEnabled}
                    <p>You're all caught up!</p>
                {:else}
                <p><Button disabled={!loadTopEnabled} on:click={() => checkMoreTop(1)}>Load 1</Button>
                    <Button disabled={!loadTopEnabled} on:click={() => checkMoreTop(10)}>Load 10</Button>
                    <Button disabled={!loadTopEnabled} on:click={() => checkMoreTop(25)}>Load 25</Button>
                    <Button disabled={!loadTopEnabled} on:click={() => checkMoreTop(50)}>Load 50</Button>
                </p>
                {/if}
                <p>
                    Go to: 
                    <Button disabled={!loadTopEnabled} on:click={goToNow}>Now</Button>
                    <Button on:click={goToDate} disabled={!dateButton}>Date</Button>
                    <input type="datetime-local" bind:this={goToDateInput} bind:value={goToDateValue}/>
                </p>
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
{/if}
<svelte:window bind:scrollY on:keyup={onWindowKeyUp} on:pageshow={onPageShow}/>

<script lang="ts">
import type { PageEvent } from "./ItemView.svelte";
import { DisplayItem, ItemOffsetParams, ItemFilter, LazyItemLoader, protobuf as pb } from "../ts/client";

import { ConsoleLogger, InfiniteScroll, Mutex } from "../ts/common";
import ItemView from "./ItemView.svelte";
import type { Writable } from "svelte/store";
import type { AppState } from "../ts/app";
import { getContext, onDestroy, tick } from "svelte";
import ItemBox from "./ItemBox.svelte";
import ShowWhen from "./widgetes/ShowWhen.svelte";
import Button from "./Button.svelte";
import { ElapsedTime } from "../ts/asyncStore";
import { query } from "svelte-hash-router"


const logger = new ConsoleLogger({prefix: "<ItemScroll>"})// .withDebug()
logger.debug("Created logger. (fresh load)")

export let createItemLoader: (offset: ItemOffsetParams) => AsyncGenerator<pb.ItemListEntry>

export let itemFilter: ItemFilter = ItemFilter.allowAll()

let appState: Writable<AppState> = getContext("appStateStore")


let loader: LazyItemLoader|undefined
type Direction = "up"|"down"
let loaderDirection: Direction = "down"
let load = {
    initial: 15,
    incremental: 5
} as const


// We've reached the end of the items at the bottom/top:
let noMoreBottom = false
let noMoreTop = false
let loadingTop = false
let showTopStatus = false
let showGoTo = false
let elapsedSinceNoMoreTop = new ElapsedTime()
$: loadTopEnabled = !noMoreTop || $elapsedSinceNoMoreTop.elapsedMs > 10_000

let goToDateInput: HTMLInputElement
let goToDateValue: number|undefined
$: dateButton = goToDateValue !== undefined

$: logger.debug("scrollPos", scrollPos)
$: logger.debug("noMoreBottom", noMoreBottom)
$: logger.debug("noMoreTop", noMoreTop)
$: nothingToShow = noMoreBottom && noMoreTop && $items.length == 0
$: showSlot = nothingToShow && $$slots.whenEmpty

function initLoaderTop() {
    logger.debug("initLoaderTop()")
    if (loader && !loader.done && loaderDirection == "up") {
        logger.debug("top loading already initialized")
        return
    }

    return reInitLoader({after: scrollPos}) 
}

function reInitLoader(offset: ItemOffsetParams) {
    // This can happen when Svelte re-uses the component:
    loader?.abort()

    let itemLoader = createItemLoader(offset)


    let direction = typeof(offset.before) == "number" ? "down" : "up"
    logger.debug("Reinit loader going", direction)
    if (direction == "down") {
        noMoreBottom = false
    } else {
        noMoreTop = false
    }

    let newLoader = new LazyItemLoader({
        client: $appState.client,
        source: itemLoader,
        filter: itemFilter,
    })

    loader = newLoader
    loaderDirection = offset?.before ? "down" : "up"
    return loader
}

let items = new InfiniteScroll<DisplayItem>()

let visibleElements: PageEvent[] = []
// $: logger.debug("visible", visibleElements.map(e => e.item?.timestamp_ms_utc))

function itemEnteredScreen(event: CustomEvent<PageEvent>) {
    visibleElements = [...visibleElements, event.detail]

    // logger.debug("Item entered page", event.detail.signature.substring(0, 10))

    let ts = event.detail.item?.timestampMsUtc
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
        let ts = e.item.timestampMsUtc
        if (event == null || ts > event.item!.timestampMsUtc) {
            event = e
        }
    }

    return event
}

$: saveScrollPosition(firstVisible)
function saveScrollPosition(event: PageEvent|null) {
    if (!event) { return }
    let ts = event.item?.timestampMsUtc
    if (!ts) { return }

    historyThrottle.setParam("ts", `${ts}`)
    scrollPos = Number(ts)
}

// Hmm, should this be auto-updating if we update it in saveScrollPosition?
// Ah, $query only gets updated when the URL is initially loaded or manually changed,
// not when we update history w/ the current scrollPos. Weird edge case but handy.
$: scrollPos = parseScrollPosition($query.ts)
$: logger.debug({scrollPos})
function parseScrollPosition(ts: string): number {
    let pos: number
    try { 
        let parsed = parseInt(ts) 
        if (isNaN(parsed)) { throw new Error("NaN")}
        pos = parsed
    } catch {
        pos = new Date().valueOf() 
    }
    return pos
}

// if user changes itemfilter, or manually navigates to new scrollPosition, reload.
$: onFilterChange(itemFilter, $query.ts)
function onFilterChange(..._changedFields: unknown[]) { 
    logger.debug("onFilterChange()")
    items.clear()

    noMoreTop = false
    noMoreBottom = false
    showTopStatus = false
    reInitLoader({before: scrollPos + 1})

    loadMore(load.initial)
}

let loadMutex = new Mutex()

async function loadMore(count: number) {
    loadMutex.runIfNone(() => _loadMore(count))
}

async function _loadMore(count: number) {
    logger.debug("loadMore(", count, ")")

    if (!loader) { 
        logger.debug("no loader, ending loadMore()")
        return
    }

    let nextItems = await loader.getNext(count)
    logger.debug("Tried to load", count, "items and got", nextItems.length)

    if (loaderDirection == "up") {
        for (let item of nextItems) { await items.pushTop(item) }
        if (loader.done) {
            noMoreTop = true
            elapsedSinceNoMoreTop.startNow()
        }
    } else {
        for (let item of nextItems) { await items.pushBottom(item) }
        if (loader.done) { noMoreBottom = true }
    }

    logger.debug("loadMore() done")
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
let shrinkWatermark: bigint|undefined = undefined

function shrinkImages(entry: DisplayItem): boolean {
    // For the very first item, don't wait for an itemEnteredScreen event
    // If it doesn't happen from scrolling down, scrolling up will not get a shrinkImage=true, which will
    // cause scrolling issues.
    let ts = entry.item.timestampMsUtc
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

    // logger.debug("onVerticalScroll previous", previousScrollY, "new", newY)
    let oldScrollY = previousScrollY
    previousScrollY = newY

    let winHeight = window.innerHeight
    // -5 because docHeight is a float and may not be precisely == in long docs (observed in Chrome)
    let docHeight = document.body.scrollHeight - 5

    // The threshold for being "near" is one screen:
    let nearHeight = winHeight

    // Note: Check bottom first because it's the preferred scroll direction:
    // If we've already loaded everything at the bottom, don't care if we're at/near there.
    if (!noMoreBottom) {
        let bottomY = newY + winHeight
        if (bottomY >= docHeight && bottomY > oldScrollY) {
            logger.debug("bumpedBottom")
            bumpedBottom()
            return
        }

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


$: if(noMoreBottom) {
    // We might not have been able to scroll down, so just show the top:
    showTopStatus = true
}



function nearTop() {
    // TODO: Delete?
    return

    // // Only when we bump the top do we try loading in that direction.
    // // But we make an exception if we started at the bottom:
    // if (!haveBumpedTop) { return }
    // logger.debug("nearTop")

    // if (!topLoader) {
    //     let topTs = $items[0]?.item?.timestamp_ms_utc
    //     if (!topTs) { return }
    //     topLoader = reInitLoader(topLoader, {after: topTs})
    //     if (shrinkWatermark) {
    //         // TODO: WHy did I think I needed to do this?
    //         // shrinkWatermark = undefined
    //     }
    // }

    // if (!topLoader?.hasMore) { return }

    // // Reset bottomLoader because we may end up truncating the list.
    // bottomLoader?.stop()
    // bottomLoader = null
    // topLoader.displayMoreItems()
}



function nearBottom() {
    if (loaderDirection != "down") {
        let myItems = $items
        let end = myItems.length - 1
        let bottomTs = myItems[end]?.item?.timestampMsUtc
        if (!bottomTs) { return }
        reInitLoader({before: Number(bottomTs)})
    }
    
    if (loader?.done) {
        return
    }

    loadMore(load.incremental)
}

// No real need to distinguish these cases:
function bumpedBottom() { nearBottom() }
function bumpedTop() { nearTop() }

async function checkMoreTop(maxItems: number) {
    logger.log("checkMoreTop()")
    if (loadingTop) { return }
    try {
        loadingTop = true
        initLoaderTop()
        await loadMore(maxItems)
    } finally {
        loadingTop = false
    }

}


//----------- Stuff for keyboard navigation --------------- 

let activeItemSignature: string|undefined = undefined

// Note, we don't bind mouseenter (above) because browsers fire that when you scroll an item into view under the mouse cursor. boo.
function onItemMouseEnter(item: DisplayItem) {
    let newSig = item.signature.asBase58
    if (activeItemSignature == newSig) { return } 
    // logger.debug("onItemMouseEnter event", newSig)
    activeItemSignature = newSig
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
        $appState.scroll.keyboardScrollTo({top: 0})
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

    top -= 25
    $appState.scroll.keyboardScrollTo({top})
}

function onPageShow(event: PageTransitionEvent) {
    logger.debug("onPageShow persisted:", event.persisted, "url", window.location.toString())

    // Safari on iOS tries to load a "persisted" version of the page but apparently doesn't persist
    // it correctly so it always goes back to the top. Force a reload to get back to our saved scroll
    // position:
    if (event.persisted) {
        window.location.reload()
    }
}

function goToNow() {
    scrollPos = new Date().valueOf()
    onFilterChange()
}

function goToDate() {
    if (!goToDateInput) {
        logger.error("Can't find goToDateInput")
        return
    }


    let value = goToDateInput.value
    if (!value) {
        logger.warn("No date selected")
        return
    }

    let date = new Date(value)
    let msUtc = date.valueOf()
    scrollPos = msUtc
    onFilterChange()
}

</script>