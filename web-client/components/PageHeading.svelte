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

$: getNav($appState)
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


// TODO: Move this up to IndexPage along side the Router config?
// TODO: Can I use the Navigator class here for URLs?
let navTree = new NavNode({
    pattern: "/home",
    title: window.location.hostname,
    children: [
        {
            requireCurrentUser: true,
            title: "My Feed",
            pattern: "/u/:uid/feed",
            linkAway: true,
        },
        {
            requireLoggedIn: true,
            title: "Log Out",
            pattern: "/login",
            linkAway: true,
        },
        {
            requireLoggedIn: false,
            title: "Log In",
            pattern: "/login",
            linkAway: true,
        },
        
        { 
            pattern: "/login",
            title: "Log In/Out",
            placeholder: true,
            children: [
                {title: "Home", pattern: "/home", linkAway: true},
                {title: "Identities", pattern: "/login" },
                {title: "Create ID", pattern: "/login/create-id" },
                {title: "Security", pattern: "/login/security", requireLoggedIn: true},
            ]
        },
        {
            requireCurrentUser: true,
            title: "My Posts",
            pattern: "/u/:uid/(*)",
            userIdParam: "uid",
            placeholder: true,
            children: [
                {title: "Home", pattern: "/home", linkAway: true},
                { pattern: "/u/:uid/", title: "Posts" },
                { pattern: "/u/:uid/profile", title: "Profile" },
                { pattern: "/u/:uid/feed", title: "Feed" },
                { pattern: "/u/:uid/post", title: "New Post"},
                // TODO: Just move into the profile view:
                { pattern: "/u/:uid/profile/edit", title: "Edit Profile"},
                { pattern: "/u/:uid/sync", title: "Sync"},
            ]
        },
        {
            requireCurrentUser: false,
            title: "User Posts",
            pattern: "/u/:uid/(*)",
            userIdParam: "uid",
            placeholder: true,
            children: [
                {title: "Home", pattern: "/home", linkAway: true},
                { pattern: "/u/:uid/", title: "Posts" },
                { pattern: "/u/:uid/profile", title: "Profile" },
                { pattern: "/u/:uid/feed", title: "Feed" },
            ]
        },
    ]
})



function getNav(app: AppState) {
    let url = window.location.hash.substring(1).replace(/\?.*/, "")
    let node = navTree.getDisplayNode(app, url)
    if (!node) {
        navItems = [{text: "Couln't determine navigation", href: "#/"} ]
        return
    }
    navItems = node.getNavItems(app, url)
    breadcrumbs = node.getBreadcrumbs(url)
}

</script>

<script lang="ts" context="module">

// TODO: Ugh, this class was not designed. It was evolved. It seems overly complicated for generating such a small
// navigation.  I'd like to rewrite the whole thing when I've got time. If you are reading this, I apologise.
//
// A way to declare our nav hierarchy and let the URL patterns figure it out:
// Note: the path key ":uid" is special.
class NavNode {
    readonly urlPattern: UrlPattern
    readonly title: string
    private parent?: NavNode
    readonly children: NavNode[]
    private userIDcrumb?: string
    private placeholder: boolean
    private requireCurrentUser: boolean | undefined;
    private requireLoggedIn: boolean | undefined;
    private linkAway: boolean | undefined;

    constructor(params: NavNodeParams) {
        this.urlPattern = new UrlPattern(params.pattern)
        this.title = params.title
        this.children = params.children?.map((c) => new NavNode(c)) ?? []
        this.children.forEach((c) => c.parent = this)
        this.userIDcrumb = params.userIdParam
        this.placeholder = params.placeholder ?? false
        this.requireCurrentUser = params.requireCurrentUser
        this.requireLoggedIn = params.requireLoggedIn
        this.linkAway = params.linkAway
    }

    // If this is chosen as the main navNode, what nav items should we show?
    getNavItems(app: AppState, url: string): NavItem[] {
        let currentUser = app.loggedInUser
        let loggedIn = !!currentUser

        let items: NavItem[] = []

        let params = this.compileParams(url)
        let uid = params.uid
        
        

        for (let child of this.children) {
            if (child.placeholder) { continue }
            if (child.requireLoggedIn !== undefined) {
                if (child.requireLoggedIn != loggedIn) continue
            }

            let childParams = {...params}
            if (child.requireCurrentUser !== undefined) {
                if (uid) {
                    let isCurrentUser = uid == currentUser?.asBase58
                    if (child.requireCurrentUser != isCurrentUser) { continue }
                } else {
                    // parent hasn't specified uid, use currentUser.
                    if (!currentUser) { continue } // there is none!
                    childParams.uid = currentUser.asBase58
                }
            }
            let childUrl = child.getUrl(url, childParams)
            items.push({
                text: child.title,
                href: '#' + childUrl,
                isActive: child.matches(app, url),
            })
        }

        return items
    }

    getBreadcrumbs(url: string): Breadcrumb[] {
        let trail = []
        for (let node: NavNode|undefined = this; node; node = node.parent) {
            trail.push(node)
        }

        let crumbs = trail.reverse().map((n) => n.getBreadCrumb(url))
        if (crumbs.length > 1 && "text" in crumbs[0]) {
            crumbs = crumbs.slice(1)
        }

        return crumbs
    }

    private getBreadCrumb(url: string): Breadcrumb {
        if (this.userIDcrumb) {
            let params = this.compileParams(url)
            let userID = params[this.userIDcrumb]
            if (!userID) { throw new Error(`no such userIDCrumb in path: ${this.userIDcrumb}`)}
            return {
                userID: UserID.fromString(userID)
            }
        } 
        return {
            text: this.title,
            href: '#' + this.getUrl(url)
        }

    }

    // Get URL path params from all of my parents, and this.
    private compileParams(url: string): Partial<Record<string,string>> {
        let baseParams = {}
        if (this.parent) {
            baseParams = this.parent.compileParams(url)
        }

        let match = this.urlPattern.match(url)
        if (!match) {
            return baseParams
        }
        return {...baseParams, ...match}
    }

    private getUrl(currentUrl: string, defaultParams?: Partial<Record<string,string>>): string {
        let params = { ...this.compileParams(currentUrl), ...defaultParams}

        return this.urlPattern.stringify(params)
    }

    /** Get the node from which to show nav & breadcrumbs */
    getDisplayNode(app: AppState, url: string): NavNode | null {
        // Depth-first search:
        let node: NavNode|null = null

        // ... unless this node is excluded for some reason:
        
        if (this.linkAway) { return null }

        let currentUserID = app.loggedInUser?.asBase58
        if (this.requireLoggedIn !== undefined) {
            let loggedIn = !!currentUserID
            if (this.requireLoggedIn != loggedIn) { return null }
        }

        if (this.requireCurrentUser !== undefined) {
            let pathArgs = this.compileParams(url)
            let uid = pathArgs.uid
            let isCurrentUser = uid && (uid == currentUserID)
            if (this.requireCurrentUser != isCurrentUser) { return null }
        }

        for (let child of this.children) {
            node = child.getDisplayNode(app, url)
            if (node != null) { break }
        }

        if (node == null && this.matches(app, url)) {
            if (this.children.length == 0 && this.parent) {
                // Go up one level to show sibling nav:
                node = this.parent
            } else {
                node = this
            }
        }

        return node
    }

    private matches(app: AppState, url: string): boolean {
        return !!this.urlPattern.match(url)
    }

}

interface NavNodeParams {
    pattern: string,
    title: string,

    /** Instead of displaying the page title, use this parameter name to pull & display the user profile by ID. */
    userIdParam?: string,

    /** This item is just a placeholder for breadcrumb nav and shouldn't show up as a child nav item. */
    placeholder?: boolean,

    children?: NavNodeParams[]

    // true: Only show this link if the user ID matches the "current" (logged-in) user ID.
    // false: Only show this when the uid is NOT the current user.
    requireCurrentUser?: boolean

    // true = Only show this if the user is "logged in"
    // false = Only show this if the user is not logged-in.
    requireLoggedIn?: boolean

    // This link should not be counted as a nav node, we only display it to link away to a different section.
    linkAway?: boolean
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

export interface Breadcrumbs {
    // The first item can be a user icon, if this is set:

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