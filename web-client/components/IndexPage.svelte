
<div class="nav-layout-container">
    <div class="nav-container">
        <div class="nav">
            {#if currentUser == null}
                <NavLink href="#/home">Home</NavLink>
                <NavLink href="#/login">Log in</NavLink>
            {:else}
                <NavLink href={$appState.navigator.userPosts(currentUser).hash}>{$appState.userName || "(unknown user)"}</NavLink>
                <!-- <NavLink href="#/u/{$appState.loggedInUser}/feed">My Feed</NavLink>
                <NavLink href="#/u/{$appState.loggedInUser}/profile">My Profile</NavLink>
                <NavLink href="#/u/{$appState.loggedInUser}/">My Posts</NavLink>
                <NavLink href="#/post">New Post</NavLink>
                <NavLink href="#/sync">Sync</NavLink> -->
                <NavLink href="#/login">Change User</NavLink>
                <NavLink href="#/home">Home</NavLink>
            {/if}
        </div>

    </div>

    <div class="items">
        <Router />
    </div>

</div>

<script context="module" lang="ts">
    import NotFoundPage from "./NotFoundPage.svelte"
    import * as app from "../ts/app"
</script>

<script lang="ts">
import { writable } from "svelte/store";
import { setContext } from "svelte";
import RootPage from "./pages/RootPage.svelte";
import NavLink from "./NavLink.svelte";
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

// This is a writable() store so that we can notify the app
// that appState has been modified. Svelte doesn't/can't propagate updates
// from simple AppState.prop = newValue type statements. Instead, you should
// use appState.update((state) => { /* ... */ }) to modify the state.
// Then consumers of the store will be rerendered.
let appState = writable(new app.AppState())
setContext("appStateStore", appState)

$: currentUser = $appState.loggedInUser


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

