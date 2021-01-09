
<div class="nav-layout-container">
    <div class="nav-container">
        <div class="nav">
            <a href="#/">Home</a>
        </div>
    </div>

    <div class="items">
        <Router {routes}/>
    </div>
</div>

<script context="module" lang="ts">
    import Router from "svelte-spa-router"

    import NavBar from "./NavBar.svelte"
    import ItemView from "./ItemView.svelte"
    import IndexPageDefault from "./IndexPageDefault.svelte"
    import NotFoundPage from "./NotFoundPage.svelte"
    import PostPage from "../post/post.svelte"
    import CreateIDPage from "../create_id/create.svelte"
    
    const routes = {
        "/": IndexPageDefault,
        "/u/:userID/i/:signature/": ItemView,
        "/post/": PostPage,
        "/create_id/": CreateIDPage,
        "*": NotFoundPage,
    }

</script>

<style>

</style>