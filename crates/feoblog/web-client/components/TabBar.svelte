<!-- a tab bar that sits atop an item. -->
<div class="tabBar">
    {#each tabs as tabName, idx (tabName)}
        <span class="tab" class:inactive={idx != activeIdx} on:click={() => setActiveIndex(idx)}>{tabName}</span>
    {/each}
</div>

<script lang="ts">

export let tabs: string[] = []

export let activeTab: string

$: activeTab = tabs[activeIdx]

let activeIdx = 0

function setActiveIndex(index: number) {
    activeIdx = index
}

</script>

<style>
.tab {
    display: inline-block;
    background-color: white;
    padding: 0.25em 1em;
    border-radius: 10px 10px 0px 0px;
    margin-left: 2px;
    margin-right: 2px;
    cursor: pointer;
    user-select: none;

    /* used in our keyframe max-height below: */
    height: 1.5em;

}

:global(.tabBar + .item) {
    margin-top: 0px;
}

.tabBar {
    /* Don't want to bump into the rounded corners of an .item */
    padding-left: 40px;
    padding-right: 40px;
}

.tab.inactive {
    color: #888;
    background-color: rgb(248, 248, 248);
    filter: blur(50%);
    box-shadow: inset 0px -3px 6px -3px rgba(0, 0, 0, 0.5);
}

</style>