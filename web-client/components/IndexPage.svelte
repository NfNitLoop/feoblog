
<div class="nav-layout-container">
    <div class="nav-container">
        <div class="nav">
            {#if !$appState.loggedIn}
                <NavLink href="#/home">Home</NavLink>
                <NavLink href="#/login">Log in</NavLink>
            {:else}
                <div>{$appState.userName || "(unknown user)"}</div>
                <NavLink href="#/u/{$appState.loggedInUser}/feed">My Feed</NavLink>
                <NavLink href="#/u/{$appState.loggedInUser}/profile">My Profile</NavLink>
                <NavLink href="#/u/{$appState.loggedInUser}/">My Posts</NavLink>
                <NavLink href="#/post">New Post</NavLink>
                <NavLink href="#/sync">Sync</NavLink>
                <NavLink href="#/login">Change User</NavLink>
                <NavLink href="#/home">Home</NavLink>
            {/if}
        </div>

    </div>

    <div class="items">
        <!-- See: https://github.com/ItalyPaleAle/svelte-spa-router/issues/234 -->
        {#key $location}
            <Router {routes} on:routeLoaded={routeLoaded}/>
        {/key}
    </div>

</div>

<script context="module" lang="ts">
    import type { RouteDefinition } from "svelte-spa-router"
    import { default as Router, location, querystring }  from "svelte-spa-router"
    
    import NotFoundPage from "./NotFoundPage.svelte"
    import * as app from "../ts/app"
</script>

<script lang="ts">
import {wrap} from "svelte-spa-router/wrap"
import { writable } from "svelte/store";
import { setContext } from "svelte";
import RootPage from "./pages/RootPage.svelte";
import NavLink from "./NavLink.svelte";

// This is a writable() store so that we can notify the app
// that appState has been modified. Svelte doesn't/can't propagate updates
// from simple AppState.prop = newValue type statements. Instead, you should
// use appState.update((state) => { /* ... */ }) to modify the state.
// Then consumers of the store will be rerendered.
let appState = writable(new app.AppState())
setContext("appStateStore", appState)


let routes = function() {
    let routes: RouteDefinition = {
        "/": RootPage,
        "/home": appPage("./pages/HomePage.svelte"),
        "/u/:userID/": appPage("./pages/UserPage.svelte"),
        "/u/:userID/feed": appPage("./pages/FeedPage.svelte"),
        "/u/:userID/profile": appPage("./pages/ProfilePage.svelte"),
        "/u/:userID/i/:signature/": appPage("./pages/ItemPage.svelte"),
        "/login": appPage('./pages/Login.svelte'),
    }

    // I tried dynamically updating `routes` based on $appState.loggedIn, 
    // but it seems like <Router> might not update its routes when they change.
    // We'll just unconditionally include these routes. We hide them from nav
    // when they're not applicable.
    Object.assign(routes, {
        "/post": appPage("./pages/PostPage.svelte"),
        "/my_profile": appPage("./pages/EditProfilePage.svelte"),
        "/sync": appPage("./pages/SyncPage.svelte")
    })

    // Must be last:
    routes["*"] = NotFoundPage
    return routes
}()


// Dynamically load a page of the app, and also pass through a reference to appState.
function appPage(templatePath: string) {

    // Snowpack now creates .svelte.js files:
    templatePath = templatePath.replace(/[.]svelte$/, ".svelte.js")

    return wrap({
        asyncComponent: () => import(templatePath),
        // TODO: Do away with manually passing appState props, and use the context instead.
        props: {
            "appState": appState,
        }
    })
}

$: {
    let color = $appState.userBGColor
    let html = (window.document.body.parentElement as HTMLElement)
    if (!color) {
        html.style.removeProperty("background-color")
    } else {
        html.style.backgroundColor = color
    }
}

$: console.log("location changed:", $location)
$: console.log("querystring changed:", $querystring)

function routeLoaded(event: unknown) {
    console.log("route loaded", event)
}

</script>


<style>

:global(body) {
    /* 
    Always show the scroll bar.
    Some things we do change the length of the page (ex: lazy-loading Items)
    and having it constantly pop in/out can be distracting.
    */
    overflow-y: scroll;
}

</style>

