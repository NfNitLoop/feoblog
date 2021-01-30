
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


import bs58 from "bs58"
import buffer from "buffer"
let Buffer = buffer.Buffer


let userID = ""

$: userIDHex = function(){
    let bytes: Buffer = bs58.decode(userID)
    return bytes.toString("hex")
}()



</script>
