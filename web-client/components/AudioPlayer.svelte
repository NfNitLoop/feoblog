<!--
    Currently, uses the in-browser player for playing .opus for FireFox and Chrome.
    In Safari, will use a WASM player.

    See: https://github.com/audiocogs/aurora.js
    and: https://github.com/audiocogs/opus.js/
-->

<!-- svelte-ignore a11y-media-has-caption -->
<div class="audioContainer">
    {#if useWASM}
        Should put the wasm player here
    {:else}
        <audio controls src={src}/>
    {/if}
</div>



<script lang="ts">
export let src: string = ""
let useWASM = detectNeedsWasm()


function detectNeedsWasm() {
    return true // TODO
}

</script>