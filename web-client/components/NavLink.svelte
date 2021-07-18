<!--
    svelte-spa-router/active can't update when the nav links are changed.
    See: https://github.com/ItalyPaleAle/svelte-spa-router/issues/187 😢
    This component works around that.
-->
<a class:active {href}><slot></slot></a>

<script lang="ts">
import { location } from "svelte-spa-router"

export let href: string

// TODO: We'll likely need to make this more permissive if I ever decide
// to try allowing ?params on hash URLs like:
// #/post?replyTo=wharrgarbl.
$: active = href === `#` + $location

</script>

<style>
.active {
    color: black;
}
</style>