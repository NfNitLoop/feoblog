<img
    src={imgSrc}
    alt="identicon"
    class:defaultSize
    class:followFont
    height={sizeNum}
    width={sizeNum}
/>

<script lang="ts">
import type { UserID } from "../ts/client";

// TODO: Eventually we'll want to look up user icons for people.

export let userID: UserID

export let size: number|"line"|undefined = undefined

$: sizeNum = typeof size == "number" ? size : undefined

$: defaultSize = size === undefined
$: followFont = size === "line"
$: imgSrc = `/u/${userID}/icon.png`

</script>

<style>
img {
    margin: 0.2rem;
    border-radius: 5px;
    box-shadow: 0px 3px 3px rgba(0,0,0,0.15);
}

/* Size used in ItemView blocks:*/
.defaultSize {
    height: 2rem;
    width: 2rem;
}

.followFont {
    /** Docs say line hight is usually 1.2em, but it depends on font.
        I'm finding 1.2 bumps my line height up, so let's do 1.1.
        Smaller sizes are difficult to align well with the text.
        In particular, the 3px shadow makes it look like the image is lower than it is.
    */
    height: 1.1em;
    width: 1.1em;
    margin: 0px;

    /* 
        without align top/bottom, it aligns to baseline, which increases line height.
        Top leaves any extra space at the bottom, which works well w/ the drop shadow.
    */
    vertical-align: top;

}

</style>