<script lang="ts">

export let checked = false
export let disabled = false
export let onGrey = false

function onClick(event: InputEvent) {
    event.stopPropagation()
    if (disabled) return
    checked = !checked
}

</script>

<check-box class:checked class:onGrey on:click={onClick}>
    <input type="checkbox" {checked}> <slot></slot>
</check-box>

<style>
check-box {
    display: inline-block;
    padding: 0.25rem 0.75rem;
    padding-left: 0.25rem;
    border-radius: 3px;
    background-color: #eee;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
    user-select: none;
}

check-box.onGrey {
    background-color: white;
}


check-box input[type="checkbox"] {
    margin: 4px;
}

check-box:hover, check-box:not(.checked) {
    background-color: rgba(0, 0, 0, 0.1);
}

check-box:not(.checked) {
    color: #888;
}

</style>