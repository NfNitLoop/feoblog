
<div class="nav-layout-container">
    <div class="nav-container">
        <div class="nav">
            <a href="#/">Home</a>
            <button on:click={popup.toggle}>{buttonText}</button>
        </div>
    </div>

    <div class="items">
        <Router {routes}/>
    </div>

</div>


<Popup title="Settings" bind:this={popup} bind:shown={popupShown}>
    Just testing
    <div>This is a div</div>
    <br><input type="text" name="foobar" placeholder="Some text">
</Popup>

<script context="module" lang="ts">
    import Router from "svelte-spa-router"

    import ItemView from "./ItemView.svelte"
    import IndexPageDefault from "./IndexPageDefault.svelte"
    import NotFoundPage from "./NotFoundPage.svelte"
    import PostPage from "../post/post.svelte"
    import CreateIDPage from "../create_id/create.svelte"
    import Popup from "./Popup.svelte"
    
    const routes = {
        "/": IndexPageDefault,
        "/u/:userID/i/:signature/": ItemView,
        "/post/": PostPage,
        "/create_id/": CreateIDPage,
        "*": NotFoundPage,
    }
</script>

<script lang="ts">

let popup
let popupShown
$: buttonText = popupShown ? "Hide" : "Show"
$: {
    console.log("popupShown", popupShown)
}

</script>

