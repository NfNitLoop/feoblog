<script lang="ts">
import { slide } from "svelte/transition"
import Button from "../Button.svelte"
import * as nacl from "tweetnacl"
import ProfileImage from "../ProfileImage.svelte"
import { UserID } from "../../ts/client";
import { encodeBase58, encodeBase58Check } from "../../ts/fbBase58";
import { tick } from "svelte";

let pairs: Pair[]  = []

let activePair: Pair|null = null

interface Pair {
    pub: string,
    priv: string,
}

async function createID() {
    activePair = null
    pairs = []

    // Avoid re-using the <img> elements, render fresh ones:
    await tick()

    let newPairs: Pair[] = []
    for(let i = 0; i < 60; i++) {
        newPairs.push(generateId())
    }

    pairs = newPairs
}

function generateId() {
    let pair = nacl.sign.keyPair()
    
    // nacl signing secret keys contain redundant information.
    // The real private part is contained just in the first 32 bytes:
    let seed: Uint8Array = pair.secretKey.slice(0, nacl.sign.seedLength)
   
    return {
        pub: encodeBase58(pair.publicKey),
        priv: encodeBase58Check(seed),
    }
}

function clicked(pair: Pair) {
    if (activePair?.pub == pair.pub) {
        activePair = null
    } else {
        activePair = pair
    }
}

function clear() {
    pairs = []
    activePair = null
}

</script>

<div class="item">
<div class="body">
<Button on:click={createID}>Create new ID</Button>

{#if pairs.length == 0}

    <p>User IDs in FeoBlog are randomly generated <a href="https://en.wikipedia.org/wiki/Public-key_cryptography">public keys</a>. This page lets you generate some new IDs.</p>

{:else}
    <p>
    Every User ID has an associated <a href="https://en.wikipedia.org/wiki/Identicon">Identicon</a> that helps users recognize it.
    Does one of these appeal to you?
    </p>

    <icons-flex>
        {#each pairs as pair}
            <div on:click={() => clicked(pair)} class:active={activePair?.pub == pair.pub} class:inactive={activePair && activePair.pub != pair.pub} >
                <ProfileImage userID={UserID.fromString(pair.pub)}/>
            </div>
        {/each}
    </icons-flex>
   
{/if}

{#if activePair}
<div transition:slide|local>
    <table>
        <tr>
            <th>Identicon:</th>
            <td class="userID"><ProfileImage userID={UserID.fromString(activePair.pub)}/></td>
        </tr>
        <tr>
            <th>User ID:</th>
            <td class="userID">{activePair.pub}</td>
        </tr>
        <tr>
            <th>Private Key:</th>
            <td class="privateKey">{activePair.priv}</td>
        </tr>
    </table>

    <p>Next steps:</p>
    <ul>
        <li>SAVE YOUR PRIVATE KEY.
            <ul>
                <li>There is no "password reset" if you lose it!</li>
                <li>The server doesn't ever know this value.</li>
            </ul>
        <li>Request access to a server
            <ul>
                <li>If your friend runs a server, ask them to follow your User ID.</li>
                <li>Or you can <a href="https://github.com/nfnitloop/feoblog/releases">run your own server</a>!</li>
            </ul>
        </li>
        <li>Log in on that server.</li>
        <li>Create a profile for your ID. This is where you set your display name.</li>
        <li>Follow friends and feeds</li>
        <li>Write your first post(s)! 😁</li>
    </ul>
    <Button on:click={clear}>Clear</Button>
</div>
{/if}

</div>
</div>

<style>
table th {
    text-align: right
}

.userID, .privateKey {
    font-family: Consolas, monospace;
}
.privateKey {
    font-weight: bold;
    color: red;
}

icons-flex {
    display: flex;
    flex-wrap: wrap;
    gap: 1em;
    margin-bottom: 1rem;
}

icons-flex :global(img) {
    transition: transform 300ms;
    cursor: pointer;
}

icons-flex :global(img:hover), icons-flex .active :global(img) {
    transform: scale(2);
}

.inactive {
    opacity: 0.25;
}

li:not(:first-child) {
    margin-top: 0.2em;
}


</style>