<div class="items">
    <Router />
</div>


<script context="module" lang="ts">
    import NotFoundPage from "./NotFoundPage.svelte"
    import * as app from "../ts/app"
</script>

<script lang="ts">
import { writable } from "svelte/store";
import { setContext } from "svelte";
import RootPage from "./pages/RootPage.svelte";
import {routes, Router, active, query} from "svelte-hash-router"
import HomePage from "./pages/HomePage.svelte";
import FeedPage from "./pages/FeedPage.svelte";
import UserPage from "./pages/UserPage.svelte";
import ProfilePage from "./pages/ProfilePage.svelte";
import ItemPage from "./pages/ItemPage.svelte";
import Login from "./pages/Login.svelte";
import PostPage from "./pages/PostPage.svelte";
import EditProfilePage from "./pages/EditProfilePage.svelte";
import SyncPage from "./pages/SyncPage.svelte";
import CreateIDPage from "./pages/login/CreateIDPage.svelte";
import SecurityPage from "./pages/login/SecurityPage.svelte";

// This is a writable() store so that we can notify the app
// that appState has been modified. Svelte doesn't/can't propagate updates
// from simple AppState.prop = newValue type statements. Instead, you should
// use appState.update((state) => { /* ... */ }) to modify the state.
// Then consumers of the store will be rerendered.
let appState = writable(new app.AppState())
setContext("appStateStore", appState)

routes.set({
    "/": RootPage,
    "/home": HomePage,
    "/u/:userID/": UserPage,
    "/u/:userID/feed": FeedPage,

    "/u/:userID/post": PostPage,
    "/u/:userID/profile/edit": EditProfilePage,
    "/u/:userID/sync": SyncPage,

    "/u/:userID/profile": ProfilePage,
    "/u/:userID/i/:signature/": ItemPage,


    "/login": Login,
    "/login/create-id": CreateIDPage,
    "/login/security": SecurityPage,

    // These are deprecated old paths.
    // TODO: Figure out a succinct way to do redirects?
    "/post": PostPage,
    "/my_profile": EditProfilePage,
    "/sync": SyncPage,

    "*": NotFoundPage,
})


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

:global(body) {
    /* 
    Always show the scroll bar.
    Some things we do change the length of the page (ex: lazy-loading Items)
    and having it constantly pop in/out can be distracting.
    */
    overflow-y: scroll;
}

</style>

