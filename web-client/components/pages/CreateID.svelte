<script lang="ts">
import Button from "../Button.svelte"
import * as nacl from "tweetnacl"
import buffer from "buffer"
import ProfileImage from "../ProfileImage.svelte"
import { UserID } from "../../ts/client";
import { encodeBase58, encodeBase58Check } from "../../ts/fbBase58";
let Buffer = buffer.Buffer


let userID = ""
let privateKey = ""

function createID() {
    let pair = nacl.sign.keyPair()
    
    // nacl signing secret keys contain redundant information.
    // The real private part is contained just in the first 32 bytes:
    let seed: Uint8Array = pair.secretKey.slice(0, nacl.sign.seedLength)
    // Validate that we can derive the pubkey from this half of the private key:
    let derived = nacl.sign.keyPair.fromSeed(seed)

    if (!equalBytes(pair.publicKey, derived.publicKey)) {
        let message = "Failed to derive public key from private seed."
        userID = message
        throw new Error(message)
    }
    
    userID = encodeBase58(pair.publicKey)

    let buf = Buffer.from(seed)

    // The password is base58check.
    // Since we derive the public key from the seed, mistyping (or incorrectly
    // pasting) the seed could silently result in signing with a wrong/invalid
    // ID. This gives us a way to check that a password is "correct", as well
    // as distinguish between a username and password.
    privateKey = encodeBase58Check(buf)  
}

function clear() {
    userID = ""
    privateKey = ""
}

function equalBytes(array1: Uint8Array, array2: Uint8Array): boolean {
    return (array1.length == array2.length) && array1.every(
        (value, index) => value == array2[index]
    )
}

</script>

<div class="item">
<div class="body">
<Button on:click={createID}>Create new ID</Button>

{#if userID}
   
    <table>
        <tr>
            <th>Identicon:</th>
            <td class="userID"><ProfileImage userID={UserID.fromString(userID)}/></td>
        </tr>
        <tr>
            <th>User ID:</th>
            <td class="userID">{userID}</td>
        </tr>
        <tr>
            <th>Password:</th>
            <td class="privateKey">{privateKey}</td>
        </tr>
    </table>

    <p>Next steps:</p>
    <ul>
        <li>SAVE YOUR PASSWORD. There is no password reset if you lose it!</li>
        <li>Ask a server admin to add you as a user. (Or ask a friend to follow your ID.)</li>
        <li>Log in</li>
        <li>Create a profile for your ID</li>
        <li>Write your first post</li>
    </ul>
    <Button on:click={clear}>Clear</Button>
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

</style>