<!-- 
    PageHeading that may include a "settings" slot. 
    TODO: Can we remove the global pageHeading class and make this just be an .item? 

-->
<svelte:window on:scroll={onScroll} />

<heading-container class:stuckAtTop class:showHeading>

<div class="pageHeading" class:stuckAtTop>

    <div class="top">
        <div class="breadcrumbs">
            
            {#each breadcrumbs as crumb, index}
                <h1>
                {#if "text" in crumb }
                    {#if crumb.href && crumb.text == "🏠"}
                        <a href={crumb.href}><SVGButton icon="home"/></a>
                    {:else if crumb.href}
                        <a href={crumb.href}>{crumb.text}</a>
                    {:else}
                        {crumb.text}
                    {/if}
                {:else}
                    <ProfileImage size="line" userID={crumb.userID}/>
                    <UserIdView userID={crumb.userID} />
                {/if}
                </h1>
                {#if index < breadcrumbs.length - 1}
                    <h1>:</h1>
                {/if}
            {/each}
        </div>
        {#if hasSettings}
        <h1 class="settingsButton">
            <SVGButton icon="search" on:click={toggleSettings} />
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
                <a href={navItem.href} class:active={navItem.isActive}>{navItem.text}</a>
            {/each}
        </div>
    {/if}
</div>
</heading-container>


<!-- Used to determine when a relatively positioned header would be off-screen. -->
<div bind:this={headingEndElement}></div>


<script lang="ts">
import { getContext, onDestroy, onMount } from "svelte"
import { slide } from "svelte/transition"
import UrlPattern from "url-pattern"

import { UserID } from "../ts/client"
import SVGButton from "./SVGButton.svelte"
import ProfileImage from "./ProfileImage.svelte";
import UserIdView from "./UserIDView.svelte";
import type { AppState } from "../ts/app";
import type { Writable } from "svelte/store";
import { CancelTimer, ConsoleLogger } from "../ts/common";

let navItems: NavItem[] = []
let breadcrumbs: Breadcrumb[] = []

let appState: Writable<AppState> = getContext("appStateStore")

let headingEndElement: HTMLElement

let logger = new ConsoleLogger({prefix: "<PageHeading>"}) //.withDebug()

// Show the heading even if it's off-screen.
let forceShow = false

// "stuck" here meaning the css sticky mode is enabled. (roughly)
let stuckAtTop = false

$: showHeading = stuckAtTop && forceShow

let observer = new IntersectionObserver(observerCallback, {threshold: [1]})
let intersectionRatio = 1;
function observerCallback(entries: IntersectionObserverEntry[], observer: IntersectionObserver) {
    // We only observe this one element, so this should always be here:
    let entry = entries[0]

    intersectionRatio = entry.intersectionRatio

    stuckAtTop = intersectionRatio < 1
}

onMount(() => {
    observer.observe(headingEndElement)
})
onDestroy(() => {
    observer.disconnect()
})

let settingsHidden = true

function toggleSettings() {
    settingsHidden = !settingsHidden
}

$: setNav($appState)
$: hasNav = navItems.length > 0
$: navHidden = !settingsHidden

$: hasSettings = !!$$slots.settings

let scrollYDelta = new ScrollDelta()

function onScroll(event: UIEvent) {
    if (event.type != "scroll") { return }
    if (!stuckAtTop) { return }
    if ($appState.scroll.scrolledViaKeyboard) { 
        // Hide the header when scrolling with keyboard:
        forceShow = false
        return
     }

    logger.debug("onScroll event", event)
    
    let delta = scrollYDelta.update()

    logger.log("onScroll delta", delta)

    // Small scroll deltas (usually <1, always <2) seem to be the browser settling after document length has changed:
    if (delta < -10) {
        forceShow = true
    } else if (delta > 10) {
        forceShow = false
    }
}

$: logger.log("forceShow", forceShow)

/**
 * Updates navItems and breadcrumbs
 */
function setNav(app: AppState) {
    const currentUser = app.loggedInUser
    const loggedIn = currentUser !== null

    // Relative path of this page inside our #/ navigation, minus ?query=strings.
    let pagePath = window.location.hash.substring(1).replace(/\?.*/, "")

    const navs: NavItem[] = []

    const onHome = new UrlPattern("/home").match(pagePath)
    // Always present:
    navs.push({
        text: "Home",
        href: "#/home",
        isActive: onHome
    })

    let uid: string|undefined = currentUser?.asBase58

    // If we're viewing a different user's page, repurpose logged-in nav to their nav:
    const userRoot = new UrlPattern("/u/:uid(/*)").match(pagePath)
    if (userRoot) { uid = userRoot.uid }
    const isMe = loggedIn && uid == currentUser.asBase58
    const my = isMe ? "My " : ""

    // Display User Nav.
    // We always display the logged-in user's nav, unless they're looking at some other user:
    if (loggedIn || userRoot) {
        navs.push({
            text: `${my}Feed`,
            href: `#/u/${uid}/feed`,
            isActive: pagePath == `/u/${uid}/feed`
        })

        navs.push({
            text: `${my}Posts`,
            href: `#/u/${uid}/`,
            isActive: new UrlPattern("/u/:uid/").match(pagePath),
        })

        navs.push({
            text: `${my}Profile`,
            href: `#/u/${uid}/profile`,
            isActive: new UrlPattern("/u/:uid/profile").match(pagePath),
        })

        // TODO: Inline "New Post" and "Edit Profile" into the Post/Profile page.
        if (isMe) {
            navs.push({
                text: `New Post`,
                href: `#/u/${uid}/post`,
                isActive: new UrlPattern("/u/:uid/post").match(pagePath),
            }) 

            navs.push({
                text: `Edit Profile`,
                href: `#/u/${uid}/profile/edit`,
                isActive: new UrlPattern("/u/:uid/profile/edit").match(pagePath),
            }) 

            navs.push({
                text: `Sync`,
                href: `#/u/${uid}/sync`,
                isActive: new UrlPattern("/u/:uid/sync").match(pagePath),
            }) 

        }
    }

    const onLogin = new UrlPattern("/login(/*)").match(pagePath)
    navs.push({
        text: loggedIn ? "Log Out" : "Log In",
        href: "#/login",
        isActive: new UrlPattern("/login").match(pagePath)
    })

    // TODO: Fold these into the "Login" page so they don't need to be separate nav:
    // For now, only show them when:
    if (onLogin && loggedIn) {
        navs.push({
            text: "Create ID",
            href: "#/login/create-id",
            isActive: pagePath == "/login/create-id",
        })
        navs.push({
            text: "Security",
            href: "#/login/security",
            isActive: pagePath == "/login/security",
        })
    }

    navItems = navs
    if (onHome) {
        breadcrumbs = [{
            text: window.location.hostname,
            href: "#/home",
        }]
    } else if (uid) {
        try {
            breadcrumbs = [{
                userID: UserID.fromString(uid)
            }]
        } catch (e) {
            breadcrumbs = [{
                text: `Invalid user ID: ${uid}`
            }]
        }
    } else if (onLogin) {
        breadcrumbs = [{
            text: "Log In"
        }]
    } else {
        breadcrumbs = [{
            text: `Unknown Page: ${window.location.hash.slice(1)}`
        }]
    }
}

</script>

<script lang="ts" context="module">

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
            return this.delta
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

// TODO: Refactor. There's only ever one.
export interface Breadcrumbs {
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
    isActive?: boolean
}

</script>


<style>



heading-container {
    display: block;
    position: sticky;
    /* Using calc here breaks animation: */
    /* top: calc(0 - var(--heading-max-height)); */
    /* Should be >= the max-height of pageHeading */
    top: -51vh;


    overflow-y: visible;
    /* Required so that transform'd items don't bleed through. Weird. */
    z-index: 1;
    transition: all 200ms;

}


heading-container.showHeading {
    top: 0;
}

.pageHeading {
    margin: 1rem;
    max-width: 55rem;
    max-height: 50vh;
    overflow-y: auto;
}

.pageHeading :global(h1) {
	margin-top: 0px;
	margin-bottom: 0px;
    transition: font-size 300ms;
}

.pageHeading.stuckAtTop {
    margin-left: 0;
    margin-right: 0;
    border-radius: 0 0 20px 20px;
}


.settings {
    padding-top: 1rem;
}




/* Must use same @media selector as in style.css: */
@media(min-width: 55em) {
    .pageHeading.stuckAtTop {
        margin: 0.5rem;
        max-width: 56rem;
    }
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
    margin-top: 0.3em;
    gap: 0.3em;
}

.navItems a {
    border-radius: 5px;
    padding: 0.2em 0.4em;
    transition-property: background-color, color;
    transition-duration: 300ms;
    background-color: #eee;
    color: #888;
}

.navItems a:hover, .navItems a.active {
    color: black;
}


.breadcrumbs::-webkit-scrollbar, .navItems::-webkit-scrollbar {
    display: none;
}
/* Firefox: */
.breadcrumbs, .navItems {
    scrollbar-width: none;
}

</style>