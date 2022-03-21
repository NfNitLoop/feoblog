<!-- 
    PageHeading that may include a "settings" slot. 
    TODO: Can we remove the global pageHeading class and make this just be an .item? 

-->
<svelte:window on:scroll={onScroll} />
<div class="pageHeading" class:stuckAtTop class:hideHeading bind:this={headingElement}>

    <div class="top">
        <div class="breadcrumbs">
            
            {#each breadcrumbs.crumbs as crumb, index}
                <h1>
                {#if "text" in crumb }
                    {#if crumb.href}
                        <a href={crumb.href}>{crumb.text}</a>
                    {:else}
                        {crumb.text}
                    {/if}
                {:else}
                    <ProfileImage size="line" userID={crumb.userID}/>
                    <UserIdView userID={crumb.userID} />
                {/if}
                </h1>
                {#if index < breadcrumbs.crumbs.length - 1}
                    <h1>&gt;</h1>
                {/if}
            {/each}
        </div>
        {#if hasSettings}
        <h1 class="settingsButton">
            <SVGButton src="/client/images/magnifying_glass.svg" alt="search" on:click={toggleSettings} />
        </h1>
        {/if}
    </div>



    {#if hasSettings && !settingsHidden}
        <div class="settings" transition:slide|local>
            <slot name="settings"></slot>
        </div>
    {/if}

    {#if hasNav && !navHidden}
        <div class="navItems" transition:slide|local>
            {#each navItems as navItem}
                <Button href={navItem.href}>{navItem.text}</Button>
            {/each}
        </div>
    {/if}
</div>

<script lang="ts">
import { onDestroy, onMount } from "svelte"
import { slide } from "svelte/transition"
import type { UserID } from "../ts/client"
import SVGButton from "./SVGButton.svelte"
import VisibilityCheck from "./VisibilityCheck.svelte"
import Button from "./Button.svelte"
import ProfileImage from "./ProfileImage.svelte";
import UserIdView from "./UserIDView.svelte";

export let navItems: NavItem[] = []
export let breadcrumbs: Breadcrumbs = { crumbs: []}

let headingElement: HTMLElement;
let stuckAtTop = false
let hideHeading = false

let observer = new IntersectionObserver(observerCallback, {threshold: [1]})
function observerCallback(entries: IntersectionObserverEntry[], observer: IntersectionObserver) {
    // We only observe this one element, so this should always be here:
    let entry = entries[0]
    stuckAtTop = entry.intersectionRatio < 1
}

onMount(() => {
    observer.observe(headingElement)
})
onDestroy(() => {
    observer.disconnect()
})

let settingsHidden = true

function toggleSettings() {
    settingsHidden = !settingsHidden
}


$: hasNav = navItems.length > 0
$: navHidden = !settingsHidden

$: hasSettings = !!$$slots.settings

let scrollYDelta = new ScrollDelta()

function onScroll(event: UIEvent) {
    if (event.type != "scroll") { return }
    if (!stuckAtTop) { return }
    
    let delta = scrollYDelta.update()

    if (delta < -10) {
        hideHeading = false
    }
    if (delta > 100) {
        scrollYDelta.delta = 100

        hideHeading = true
    }
}

</script>

<script lang="ts" context="module">

class CancelTimer {
    delayMs = 5000

    private timer: number|null = null

    start(callback: () => unknown) {
        this.cancel()
        this.timer = setTimeout(callback, this.delayMs)
    }

    cancel() {
        if (this.timer) {
            clearTimeout(this.timer)
        }
        this.timer = null
    }
}

class ScrollDelta {
    delta = 0

    private lastScrollY = 0
    private lastDocLength = 0
    private deltaTimer = new CancelTimer()

    constructor() {
        this.deltaTimer.delayMs = 200
    }

    update(): number {
        let newScrollY = window.scrollY

        let newDocLength = document.body.scrollHeight
        if (newDocLength != this.lastDocLength) {
            // Just ignore this event, stuff got resized.
            this.lastScrollY = newScrollY
            this.lastDocLength = newDocLength
        }

        let newDelta = newScrollY - this.lastScrollY
        this.lastScrollY = newScrollY

        this.delta += newDelta
        this.deltaTimer.start(() => {
            this.delta = 0
        })

        return this.delta
    }
}

export interface Breadcrumbs {
    // The first item can be a user icon, if this is set:
    userID?: UserID

    crumbs: Breadcrumb[]
}

export type Breadcrumb = TextBreadcrumb | UserBreadcrumb

export interface TextBreadcrumb {
    text: string
    href?: string
}

export interface UserBreadcrumb {
    userID: UserID
}

export interface NavItem {
    text: string
    href: string
}

</script>


<style>
.pageHeading {
    margin: 1rem;
	position: sticky;
    top: -1px;
    transition: all 300ms;
    max-width: 55rem;
    max-height: 50vh;
    overflow-y: auto;
    /* Required so that transform'd items don't bleed through. Weird. */
    z-index: 1;
}

.pageHeading :global(h1) {
	margin-top: 0px;
	margin-bottom: 0px;
    transition: font-size 300ms;
}

.pageHeading.stuckAtTop {
    padding: 0.5rem 1.3rem;
    border-top-left-radius: 0px;
    border-top-right-radius: 0px;
}

.pageHeading.stuckAtTop :global(h1) {
    font-size: 1rem;
}

.settings {
    padding-top: 1rem;
}

.pageHeading.stuckAtTop {
    margin-left: 0;
    margin-right: 0;
    border-radius: 0;
}



/* Must use same @media selector as in style.css: */
@media(min-width: 55em) {
    .pageHeading.stuckAtTop {
        margin: 0.5rem;
        max-width: 56rem;
        border-radius: 0 0 20px 20px;
    }
}

.pageHeading.stuckAtTop.hideHeading {
    top: -51vh;
}

.top {
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.navItems, .breadcrumbs {
    display: flex;
    gap: 0.3rem;
    overflow-x: scroll;
    white-space: nowrap;
    -webkit-overflow-scrolling: touch;

}

.navItems {
    margin-top: 0.5em;
}


.breadcrumbs::-webkit-scrollbar, .navItems::-webkit-scrollbar {
    display: none;
}

</style>