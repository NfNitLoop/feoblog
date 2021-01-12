import type { Item, Profile } from "../protos/feoblog"
import { Client, UserID } from "./client"

let instance: AppState|null = null

export function getInstance(): AppState {
    if (!instance) {
        instance = new AppState()
    }

    return instance
}


// A class that maintains client side application state.
export class AppState
{
    private _savedLogins: SavedLogin[] = []
    private _client: Client

    constructor() {
        this.loadSavedLogins()
        this._client = new Client({
            base_url: ""
        })
    }

    get client(): Client {
        return this._client
    }

    get loggedIn(): boolean {
        return this.loggedInUser != null
    }

    get loggedInUser(): UserID|null {
        let userID = this._savedLogins[0]?.userID
        if (!userID) return null;
        return UserID.fromString(userID)
    }

    get userBGColor(): string|null {
        return this._savedLogins[0]?.bgColor || null
    }

    // Name of the logged-in user:
    get userName(): string|null {
        let login = this._savedLogins[0]
        return login?.displayName || null
    }

    forgetLogin(userID: string) {
        let login = this.removeLogin(userID)
        this.writeSavedLogins()
        return login
    }

    // Removes the login from our list, DOES NOT save. (forgetLogin() does.)
    private removeLogin(userID: string): SavedLogin|null {
        let foundIndex = this._savedLogins.findIndex(
            (it) => it.userID == userID
        )
        if (foundIndex >= 0) {
            let login = this._savedLogins[foundIndex]
            // Odd way to remove an element:
            this._savedLogins.splice(foundIndex, 1)
            return login
        }

        return null
    }

    // Logins that the client knows about.
    // The first one is alwasy the ID that the client is currently "logged in" as.
    get savedLogins(): SavedLogin[] {
        return [...this._savedLogins]
    }

    // Save a new login (as the first item), OR, move it to the top if it already exists.
    logIn(newLogin: SavedLogin) {
        this.removeLogin(newLogin.userID)
        this._savedLogins.unshift(newLogin)
        this.writeSavedLogins()
    }

    // Update a savedLogin in place w/ a new value.
    updateSavedLogin(savedLogin: SavedLogin) {
        let foundIndex = this._savedLogins.findIndex(
            (it) => it.userID == savedLogin.userID
        )
        if (foundIndex < 0) {
            throw `No saved login for userID: ${savedLogin.userID}`
        }

        this._savedLogins[foundIndex] = savedLogin
        this.writeSavedLogins()
    }

    private writeSavedLogins() {
        try {
            let json = JSON.stringify(this._savedLogins)
            window.localStorage.setItem("savedLogins", json)
        } catch (exception) {
            console.error("Couldn't save saved logins", exception)
        }

        // Always re-load from the serialized version so lossiness is visible ASAP.
        // Also reinitializes logged-in state.
        this.loadSavedLogins()
    }

    private loadSavedLogins() {
        try {
            let json = window.localStorage.getItem("savedLogins")
            if (json === null) {
                this._savedLogins = []
                return
            }
            let logins = JSON.parse(json)
            // TODO: Some validation here?
            this._savedLogins = logins

        } catch (exception) {
            console.error("Couldn't load saved logins", exception)
        }
    }


}

// Login information we save in local browser storage.
// Needs to be JSON serializable/deserializable 
export class SavedLogin
{
    // base58-encoded user ID.
    userID: string

    displayName?: string

    // A background color like: #ff0000
    bgColor?: string
}