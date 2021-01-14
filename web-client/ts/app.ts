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

    private profileService: ProfileService

    constructor() {
        this._client = new Client({
            base_url: ""
        })
        this.profileService = new ProfileService(null, this._client)
        this.loadSavedLogins()
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

    // Calculate the preferred display name for a given user ID. 
    // Display names are calculated in this way:
    // * If the ID is the logged-in user, use their profile display_name.
    // * If the ID is followed by the logged-in user, and they specify a display_name, use that.
    // * If the ID has a profile that we can fetch from this server, use that.
    // * Otherwise, return null, this user has no preferred name.
    async getPreferredName(userID: UserID): Promise<string|null> {
        return await this.profileService.lookup(userID)
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

    // Load saved login IDs and (re)init logged-in state.
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

            this.initLoggedIn()

        } catch (exception) {
            console.error("Couldn't load saved logins", exception)
        }
    }

    // initialize sate for a logged-in user:
    private initLoggedIn() {
        this.profileService.userID = this.loggedInUser
    }
}

// TODO: Rename to NameService
// Used to resolve UserIDs to displayNames.
// See notes on AppState.getPreferredName()
class ProfileService
{
    private client: Client
    private _userID: UserID|null

    // Cache of displayNames the logged-in user has specified in their
    // profile.
    private userCache: Promise<Map<string,string>>

    // Cache of users names as specified by their own profiles
    // TODO: 
    // * Replace with an LRU or something so this doesn't grow forever?
    private globalCache: Map<string, Promise<string|null>> = new Map()

    constructor(loggedInUser: UserID|null, client: Client) {
        this.client = client
        this.userID = loggedInUser
    }

    set userID(userID: UserID|null) {
        this._userID = userID
        this.userCache = this.getUserCache(userID)
    }

    async lookup(userID: UserID): Promise<string|null> {
        let uc = await this.userCache

        let key = userID.toString()
        let name = uc.get(key)
        if (name) {
            return name
        }

        let promise = this.globalCache.get(key)
        if (!promise) {
            promise = this.getDisplayName(userID)
            this.globalCache.set(key, promise)
        }

        return await promise
    }

    private async getDisplayName(userID: UserID): Promise<string|null> {
        let response = await this.client.getProfile(userID)
        if (!response) return null
        return response.item.profile.display_name.trim() || null
    }

    private async getUserCache(userID: UserID|null): Promise<Map<string,string>> {
        if (userID === null) {
            return new Map()
        }

        // TODO: try & log
        let result
        try {
            result = await this.client.getProfile(userID)
        } catch (e) {
            console.error(`NameService: Error fetching user profile ${userID}`, e)
            return new Map()
        }
        if (result === null || !result.item.profile) {
            // Couldn't find a profile for this user.
            console.warn(`NameService: Couldn't find a profile for logged-in user: ${userID}`)
            return new Map()
        }
        let profile = result.item.profile

        let map = new Map()
        for (let follow of profile.follows) {
            if (follow.display_name) {
                let id = UserID.fromBytes(follow.user.bytes)
                map.set(id.toString(), follow.display_name)
            }
        }

        if (profile.display_name) {
            map.set(userID.toString(), profile.display_name )
        }

        return map
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