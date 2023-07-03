<!--
    A "chip" element w/ userID, and their icon.
-->

<user-chip>
    <ProfileImage {userID} size="line" />
    <UserIDView {userID} {displayName} {href} {shouldLink} />
</user-chip>

<script lang="ts">
import type { UserID } from "feoblog-client";
import UserIDView from "./UserIDView.svelte"
    import ProfileImage from "./ProfileImage.svelte";

export let userID: UserID
export let displayName: string = ""
export let href: string|undefined = undefined
export let shouldLink = true

</script>

<style>
user-chip {
    display: inline-block;
    border-radius: 5px;
    /* TODO: Same as others */
    background-color: #eee;
    padding: 0.25em;
    margin: 0.25em;
    white-space: nowrap;
    max-width: 20em;
    text-overflow: ellipsis;
}


</style>