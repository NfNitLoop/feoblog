<!-- Simplify some of the headaches of dealing with this cruft. :p -->
<!-- bleh.  Slots don't work like I want them to.  Can't style them here. -->
<div class="item">
    <!-- {#if $$slots.header}
        <slot name="header"></slot>
    {:else if $$slots.headerLeft || $$slots.headerMid || $$slots.headerRight}
        <div class="header">
            {#if $$slots.headerLeft} <slot class="xyz" name="headerLeft"></slot> {/if}
            {#if $$slots.headerMid} <slot name="headerMid"></slot> {/if}
            {#if $$slots.headerRight} <slot name="headerRight"></slot> {/if}
        </div>
    {/if} -->
    <div class="body"><slot></slot></div>
</div>


