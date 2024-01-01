<a class:active {href}><slot></slot></a>

<svelte:window on:hashchange={hashChange} />


<script lang="ts">
export let href: string

let hashPath = nonQueryHash()
$: active = href === hashPath

function hashChange() {
    hashPath = nonQueryHash()
}

function nonQueryHash() {
    return window.location.hash.split("?")[0]
}

</script>

<style>
.active {
    color: black;
}
</style>