// Aww, FireFox/Safari don't support modules:
// See: https://bugzilla.mozilla.org/show_bug.cgi?id=1247687
// So we just use old-style JS here:

importScripts("./tweetnacl.js")

let functions  = {
    "sign_detached_verify": nacl.sign.detached.verify
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
