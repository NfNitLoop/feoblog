<!-- 
    PageHeading that may include a "settings" slot. 
    TODO: Can we remove the global pageHeading class and make this just be an .item? 

-->
<svelte:window on:scroll={onScroll} />

<heading-container class:stuckAtTop class:hideHeading>

<div class="pageHeading" class:stuckAtTop bind:this={headingElement}>

    <div class="top">
        <div class="breadcrumbs">
            
            {#each breadcrumbs as crumb, index}
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
                {#if index < breadcrumbs.length - 1}
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
                <a href={navItem.href} class:active={navItem.isActive}>{navItem.text}</a>
            {/each}
        </div>
    {/if}
</div>
</heading-container>

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

let navItems: NavItem[] = []
let breadcrumbs: Breadcrumb[] = []


let appState: Writable<AppState> = getContext("appStateStore")


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

let navTree = new NavNode({
    pattern: "/home",
    title: window.location.hostname,
    children: [
        // TODO:  Log in/out/settings/etc.
        {
            title: "User Posts",
            pattern: "/u/:userID/(*)",
            userIDcrumb: "userID",
            placeholder: true,
            children: [
                { pattern: "/u/:userID/", title: "Posts" },
                { pattern: "/u/:userID/profile", title: "Profile" },
                { pattern: "/u/:userID/feed", title: "Feed" },
                { loginState: "current-user", pattern: "/u/:userID/post", title: "New Post"},
                { loginState: "current-user", pattern: "/u/:userID/sync", title: "Sync"},
            ]
        },
    ]
})

getNav()


function getNav() {
    let app = $appState
    let url = window.location.hash.substring(1).replace(/\?.*/, "")
    let node = navTree.getDisplayNode(url)
    if (!node) {
        navItems = [{text: "Error loading nav items", href: "#/"} ]
        return
    }
    navItems = node.getNavItems(app, url)
    breadcrumbs.crumbs = node.getBreadcrumbs(url)
}

</script>

<script lang="ts" context="module">

    
// A way to declare our nav hierarchy and let the URL patterns figure it out:
// Note: the path key ":userID" is special if it matches the currently-logged-in user.
class NavNode {
    readonly urlPattern: UrlPattern
    readonly title: string
    private parent?: NavNode
    readonly children: NavNode[]
    private loginState?: LoginState
    private userIDcrumb?: string
    private placeholder: boolean

    constructor({pattern, title, loginState, children = [], userIDcrumb, placeholder = false}: NavNodeParams) {
        this.urlPattern = new UrlPattern(pattern)
        this.title = title
        this.children = children.map((c) => new NavNode(c))
        this.children.forEach((c) => c.parent = this)
        this.loginState = loginState
        this.userIDcrumb = userIDcrumb
        this.placeholder = placeholder
    }

    getNavItems(app: AppState, url: string): NavItem[] {
        let loginState: LoginState = "logged-out"
        let currentUser = app.loggedInUser
        let params = this.compileParams(url)
        if (currentUser) {
            loginState = "logged-in"
            if (currentUser.toString() == params?.userID) {
                loginState = "current-user"
            }
        }

        let items: NavItem[] = []
        for (let child of this.children) {
            if (child.placeholder) { continue }
            if (child.loginState == "current-user" && loginState != "current-user") { continue }
            if (child.loginState == "logged-in" && loginState == "logged-out") { continue }
            if (child.loginState == "logged-out" && loginState != "logged-out") { continue }
            items.push({
                text: child.title,
                href: '#' + child.getUrl(url),
                isActive: child.matches(url),
            })
        }

        return items
    }

    getBreadcrumbs(url: string): Breadcrumb[] {
        let crumbs: Breadcrumb[] = []
        if (this.parent) {
            crumbs = this.parent.getBreadcrumbs(url)
        }

        if (this.userIDcrumb) {
            let params = this.compileParams(url)
            let userID = params[this.userIDcrumb]
            if (!userID) { throw new Error(`no such userIDCrumb in path: ${this.userIDcrumb}`)}
            crumbs.push({
                userID: UserID.fromString(userID)
            })
        } else {
            crumbs.push({
                text: this.title,
                href: '#' + this.getUrl(url)
            })
        }

        return crumbs
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

    getUrl(currentUrl: string): string {
        let params = this.compileParams(currentUrl)
        console.log(currentUrl, this.urlPattern.toString(), "params:", params)

        return this.urlPattern.stringify(params)
    }

    /** Get the node from which to show nav & breadcrumbs */
    getDisplayNode(url: string): NavNode | null {
        // Depth-first search:
        let node: NavNode|null = null

        for (let child of this.children) {
            node = child.getDisplayNode(url)
            if (node != null) { break }
        }

        if (node == null && this.matches(url)) {
            if (this.children.length == 0 && this.parent) {
                // Go up one level to show sibling nav:
                node = this.parent
            } else {
                node = this
            }
        }

        return node
    }

    matches(url: string): boolean {
        return this.urlPattern.match(url)
    }

}

// When should nav be shown?
type LoginState = "current-user" | "logged-out" | "logged-in"



interface NavNodeParams {
    pattern: string,
    title: string,

    /** Instead of displaying the page title, use this parameter name to pull & display the user profile by ID. */
    userIDcrumb?: string,

    /** This item is just a placeholder for breadcrumb nav and shouldn't show up as a child nav item. */
    placeholder?: boolean,

    loginState?: LoginState
    children?: NavNodeParams[]
}

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
    top: -1px;
    overflow-y: visible;
    /* Required so that transform'd items don't bleed through. Weird. */
    z-index: 1;
    transition: all 300ms;

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

heading-container.stuckAtTop.hideHeading {
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
    margin-top: 0.1em;
}

.navItems {
    gap: 0.3em;
}

.navItems a {
    border-radius: 5px;
    padding: 0.2em 0.4em;
    transition-property: background-color, color;
    transition-duration: 300ms;
}

.navItems a:hover, .navItems a.active {
    background-color: #eee;
    color: black;
}


.breadcrumbs::-webkit-scrollbar, .navItems::-webkit-scrollbar {
    display: none;
}

</style>