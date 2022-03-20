<!-- 
    PageHeading that may include a "settings" slot. 
    TODO: Can we remove the global pageHeading class and make this just be an .item? 

-->
<VisibilityCheck hasHeight={false} bind:visible={observerVisible} />
<div class="pageHeading" class:atTop on:mouseenter={() => leftHandler.mouseEntered()} on:mouseleave={() => leftHandler.mouseLeft()}>

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
    {:else if hasNav && !navHidden}
        <div class="navItems">
            {#each navItems as navItem}
                <Button href={navItem.href}>{navItem.text}</Button>
            {/each}
        </div>
    {/if}
</div>



<script lang="ts">
import { onDestroy } from "svelte"
import { slide } from "svelte/transition"
import type { UserID } from "../ts/client"
import SVGButton from "./SVGButton.svelte"
import VisibilityCheck from "./VisibilityCheck.svelte"
import Button from "./Button.svelte"
import ProfileImage from "./ProfileImage.svelte";
import UserIdView from "./UserIDView.svelte";

/** True when the page has been scrolled down, so the bar should float to the top.*/
export let atTop = false
export let navItems: NavItem[] = []
export let breadcrumbs: Breadcrumbs = { crumbs: []}


$: hasNav = navItems.length > 0
let navHidden = false

let observerVisible = true
$: atTop = !observerVisible

$: hasSettings = !!$$slots.settings

let leftHandler = new MouseLeftHandler(() => { settingsHidden = true })
onDestroy(() => {
    leftHandler.cleanup()
})

let settingsHidden = true

function toggleSettings() {
    settingsHidden = !settingsHidden
}

</script>

<script lang="ts" context="module">

/**
 * Perform some action after the mouse has been gone for some time.
 */
class MouseLeftHandler {
    constructor(private callback: () => void) {}
    public delayMS = 5000

    private timer: number|null = null

    mouseEntered() {
        this.cleanup()
    }

    cleanup() {
        if (this.timer) {
            clearTimeout(this.timer)
            this.timer = null
        }
    }

    mouseLeft() {
        if (this.timer) {
            console.warn("MouseLeftHandler: a timer is already running? Hmm..")
            return
        }

        this.timer = setTimeout(() => {this.timerDone()}, this.delayMS)
    }

    private timerDone() {
        const callback = this.callback
        try {
            callback()
        } catch(error) {
            console.error("Error in MouseLeftHandler.callback:", error)
        }
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
    top: 0px;
    transition: all 300ms;
    max-width: 55rem;
    /* Required so that transform'd items don't bleed through. Weird. */
    z-index: 1;
}

.pageHeading :global(h1) {
	margin-top: 0px;
	margin-bottom: 0px;
    transition: font-size 300ms;
}

.pageHeading.atTop {
    padding: 0.5rem 1.3rem;
    border-top-left-radius: 0px;
    border-top-right-radius: 0px;
}

.pageHeading.atTop :global(h1) {
    font-size: 1rem;
}

.settings {
    padding-top: 1rem;
}

.pageHeading.atTop {
    margin-left: 0;
    margin-right: 0;
    border-radius: 0;
}

/* Must use same @media selector as in style.css: */
@media(min-width: 55em) {
    .pageHeading.atTop {
        margin: 0.5rem;
        max-width: 56rem;
        border-radius: 0 0 20px 20px;
    }
}

.top {
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.navItems {
    display: flex;
    gap: 0.3rem;
    margin-top: 0.5rem;
}

.breadcrumbs {
    display: flex;
    gap: 0.3rem;
}

</style>