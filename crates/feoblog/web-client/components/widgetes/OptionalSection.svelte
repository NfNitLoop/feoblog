<!--
    Checkbox with an optional section associated with it.
-->

<CheckBox bind:checked {disabled}>{buttonText}</CheckBox>

<optional-section class:shown>
    <slot></slot>
</optional-section>

<script lang="ts">
import CheckBox from "../CheckBox.svelte";


export let buttonText: string
export let checked: boolean
export let disabled: boolean = false

$: shown = checked

</script>

<style>
optional-section {
    display: none;
}

optional-section.shown {
    display: block;
    margin: 1em;
}

</style>