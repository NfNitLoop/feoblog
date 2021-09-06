<img
    src={imgSrc}
    alt="identicon"
    class:defaultSize
    height={size}
    width={size}
/>

<script lang="ts">
import type { UserID } from "../ts/client";

// TODO: Eventually we'll want to look up user icons for people.

export let userID: UserID

export let size: number|undefined = undefined
$: defaultSize = size === undefined
$: imgSrc = `/u/${userID}/icon.png`

</script>

<style>
img {
    margin: 0.2rem;
    border-radius: 5px;
    box-shadow: 0px 3px 3px rgba(0,0,0,0.15);
}

.defaultSize {
    height: 2rem;
    width: 2rem;
}

</style>