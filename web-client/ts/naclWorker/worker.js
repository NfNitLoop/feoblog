// Aww, FireFox doesn't support modules:
// See: https://bugzilla.mozilla.org/show_bug.cgi?id=1247687
import * as nacl from "tweetnacl-ts"

let functions  = {
    "sign_detached_verify": nacl.sign_detached_verify
}

onmessage = function(event) {
    let data = event.data
    let fnName = data[0]
    let args = data.slice(1)

    let fn = functions[fnName]

    if (!fn) {
        throw `Unknown function name: ${fnName}`
    }

    let result = fn(...args)
    postMessage(result)
}
