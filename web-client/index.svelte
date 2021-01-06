<div>
    
<NavBar/>

<div class="content">
<Route>
    <Route path="/" fallback>
        <p>This is a work in progress. For now, you can:</p>
        <nav>
            <a href="/create_id/">Create an ID</a>
            <a href="profile">Edit your profile</a>
            <a href="post">Write a post</a>
        </nav>
    </Route>
    <Route path="/create_id">
        <p>Create ID here.</p>
    </Route>
</Route>
</div>


</div>

<script context="module">
import {router} from "tinro"

router.mode.hash()

console.log("router", router)
</script>

<script lang="ts">
import {Route} from "tinro"
import NavBar from "./components/NavBar.svelte";


</script>

