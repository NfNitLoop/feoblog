
<div>
    <NavBar/>
    <div class="content">
    <Route>
        <Route path="/">
            <p>This is a work in progress. For now, you can:</p>
            <nav>
                <a href="/create_id/foo">Create an ID</a>
                <a href="profile">Edit your profile</a>
                <a href="post">Write a post</a>
                <a href="/u/Aci9P5BFoP27dH2pU7kVjWVaVqe7rNFnEyWeJm8ewWby/i/zDmH4ufNM9T1yNzxKqh9rMuB2i482qYgbUvnwPGcyjFL7fGW93iwRTnqmWx4LbnnDK6rR1tstzf3oVi3fbKcHim/">Sample post</a>
            </nav>
        </Route>
        <Route path="/create_id">
            <p>Create ID here.</p>
        </Route>
        <Route path="/u/:user_id/i/:signature/" let:params>
            <ItemView user_id={params.user_id} signature={params.signature}/>
        </Route>
        <Route fallback>
            Page not found. <a href="/">Home Page</a>
        </Route>
    </Route>
    </div>

</div>

<script context="module" lang="ts">
    import {Route, router} from "tinro"
    import NavBar from "./NavBar.svelte"
    import ItemView from "./ItemView.svelte"
    
    router.mode.hash()
</script>

<script lang="ts">
import { Item, Post } from "../protos/feoblog"


</script>

<style>
    nav a {
        display: block;
    }
</style>