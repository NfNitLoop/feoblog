<script lang="ts">
import bs58 from "bs58"
import * as nacl from "tweetnacl-ts"

let userID = ""
let privateKey = ""

// nacl signing secret keys contain redundant information.
// The real private part is contained just in the first 32 bytes:
const SEED_BYTES = 32;

function click() {
    let pair = nacl.sign_keyPair()
    
    let seed = pair.secretKey.slice(0, SEED_BYTES)
    // Validate that we can derive the pubkey from this half of the private key:
    let derived = nacl.sign_keyPair_fromSeed(seed)

    if (!equalBytes(pair.publicKey, derived.publicKey)) {
        let message = "Failed to derive public key from private seed."
        userID = message
        throw new Error(message)
    }
    
    userID = bs58.encode(pair.publicKey)
    privateKey = bs58.encode(seed)  
}

function equalBytes(array1: Uint8Array, array2: Uint8Array): boolean {
    return (array1.length == array2.length) && array1.every(
        (value, index) => value == array2[index]
    )
}

</script>

<div><button on:click={click}>Generate</button></div>
<table>
    <tr>
        <th>User ID:</th>
        <td class="userID">{userID}</td>
    </tr>
    <tr>
        <th>Password:</th>
        <td class="privateKey">{privateKey}</td>
    </tr>
</table>


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