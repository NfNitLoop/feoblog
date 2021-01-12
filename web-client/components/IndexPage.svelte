
<div class="nav-layout-container">
    <div class="nav-container">
        <div class="nav">
            <a href="#/">Home</a>
            {#if !$appState.loggedIn}
                <a href="#/login">Log in</a>
                <a href="#/create_id">Create an ID</a>
            {:else}
                <div>{$appState.userName || "(unknown user)"}</div>
                <a href="#/login">Change User</a>
                <a href="#/my_profile">My Profile</a>
                <a href="#/feed">My Feed</a>
                <a href="#/post">New Post</a>
                <a href="#/sync">Sync</a>
            {/if}
        </div>

    </div>

    <div class="items">
        <Router {routes}/>
    </div>

</div>

<script context="module" lang="ts">
    import Router from "svelte-spa-router"

    import NotFoundPage from "./NotFoundPage.svelte"
    import * as app from "../ts/app"
</script>

<script lang="ts">
import {wrap} from "svelte-spa-router/wrap"
import { writable } from "svelte/store";
let appState = writable(new app.AppState())

    
let routes = {
    "/": appPage("./IndexPageDefault.svelte"),
    "/u/:userID/i/:signature/": appPage("./ItemView.svelte"),
    "/post/": appPage("../post/post.svelte"),
    "/create_id/": appPage("./pages/CreateID.svelte"),
    "/login": appPage('./pages/Login.svelte'),
    "*": NotFoundPage,
}

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

