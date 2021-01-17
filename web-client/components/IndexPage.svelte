
<div class="nav-layout-container">
    <div class="nav-container">
        <div class="nav">
            {#if !$appState.loggedIn}
                <a use:active href="#/">Home</a>
                <a use:active href="#/login">Log in</a>
            {:else}
                <div>{$appState.userName || "(unknown user)"}</div>
                <a use:active href="#/u/{$appState.loggedInUser}/feed">My Feed</a>
                <a use:active href="#/my_profile">My Profile</a>
                <a use:active href="#/post">New Post</a>
                <a use:active href="#/sync">Sync</a>
                <a use:active href="#/login">Change User</a>
                <a use:active href="#/">Home</a>
            {/if}
        </div>

    </div>

    <div class="items">
        <Router {routes}/>
    </div>

</div>

<script context="module" lang="ts">
    import type { RouteDefinition } from "svelte-spa-router"
    import active from "svelte-spa-router/active"
    import Router from "svelte-spa-router"

    import NotFoundPage from "./NotFoundPage.svelte"
    import * as app from "../ts/app"
</script>

<script lang="ts">
import {wrap} from "svelte-spa-router/wrap"
import { writable } from "svelte/store";
let appState = writable(new app.AppState())

$: routes = function() {
    let routes: RouteDefinition = {
        "/": appPage("./pages/HomePage.svelte"),
        "/u/:userID/": appPage("./pages/UserPage.svelte"),
        "/u/:userID/feed": appPage("./pages/FeedPage.svelte"),
        "/u/:userID/profile": appPage("./pages/ProfilePage.svelte"),
        "/u/:userID/i/:signature/": appPage("./ItemView.svelte"),
        "/login": appPage('./pages/Login.svelte'),
    }
    if ($appState.loggedIn) {
        Object.assign(routes, {
            "/post": appPage("./pages/PostPage.svelte"),
            "/my_profile": appPage("./pages/EditProfilePage.svelte"),
        })
    }

    // Must be last:
    routes["*"] = NotFoundPage
    return routes
}()


// Dynamically load a page of the app, and also pass through a reference to appState.
function appPage(templatePath: string) {

    // Weird. with wrap(), we can import Foo.svelte, but with appPage(), 
    // it needs .js.
    templatePath = templatePath.replace(/[.]svelte$/, ".js")

    return wrap({
        asyncComponent: () => import(templatePath),
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

</script>

<style>
    .active {
        color: black;
    }
</style>