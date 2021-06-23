// The JavaScript client for FeoBlog has many more features and a better design.
// This script will redirect you there if your browser supports JavaScript.


const patterns = [
    /^[/]?$/,
    /^\/u\/[^/]+\/$/,
    /^\/u\/[^/]+\/feed\/$/,
    /^\/u\/[^/]+\/profile\/$/,
    /^\/u\/[^/]+\/i\/[^/]+\/$/,
]

function clientRedirect() {
    console.log("JavaScript-enabled browser detected.")
    let search = new URLSearchParams(window.location.search)
    let client = search.get("client")

    let sessionStorage = window.sessionStorage
    if (client === null) {
        // Fall back to sessionStorage  
        if (sessionStorage) {
            client = sessionStorage.getItem("client")
        }

        // Fall back to default:
        if (client === null) { client = "yes" }
    }

    // Store our client value:
    if (sessionStorage) {
        sessionStorage.setItem("client", client)
    }

    console.log(`client=${client}`)
    let useClient = (client == "yes") || (client == "1") || (client == "true")

    let clientBase = "/client/#"
    let path = window.location.pathname

    for (let pattern of patterns) {
        if (pattern.exec(path)) {
            let newURL = clientBase + path
            
            if (useClient) { window.location = newURL }
            else { console.log(`Would redirect to: ${newURL}`) }
            return
        }
    }

    console.log("Would not redirect.")

}

clientRedirect()
