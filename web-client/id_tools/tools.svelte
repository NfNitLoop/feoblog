
<div>
<table>
    <tr>
        <th>User ID:</th>
        <td><input bind:value={userID}></td>
    </tr>
    <tr>
        <th>Hex User ID:</th>
        <td class="userID">{userIDHex}</td>
    </tr>
</table>
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

<script lang="ts">
import { decodeBase58 } from "../ts/fbBase58";

import { Buffer } from "buffer"




let userID = ""

$: userIDHex = function(){
    let bytes = Buffer.from(decodeBase58(userID))
    return bytes.toString("hex")
}()



</script>
