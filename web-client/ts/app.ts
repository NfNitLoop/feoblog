import type { Item, Profile } from "../protos/feoblog"
import { Client, Signature, UserID } from "./client"
import { Mutex } from "./common"

let instance: AppState|null = null


// A class that maintains client side application state.
export class AppState
{
    private _savedLogins: SavedLogin[] = []
    private _client: Client

    private profileService: ProfileService

    readonly navigator = new Navigator()
    readonly scroll = new WindowScroller()
    private _loggedInUser: UserID | null = null

    constructor() {
        this._client = new Client({
            base_url: ""
        })
        this.profileService = new ProfileService(null, this._client)
        this.loadSavedLogins()
        this.loadLoggedIn()
    }

    get client(): Client {
        return this._client
    }

    get loggedIn(): boolean {
        return this.loggedInUser != null
    }

    get loggedInUser(): UserID|null {
        return this._loggedInUser
    }

    // Like loggedInUser, but throws an exception if no one is "logged in":
    requireLoggedInUser(): UserID {
        if (!this.loggedInUser) throw `Must be logged in.`
        return this.loggedInUser
    }

    get userBGColor(): string|null {
        return this.activeLogin?.bgColor || null
    }

    private get activeLogin(): SavedLogin|null {
        if (this._loggedInUser == null) { return null }
        let matches = this._savedLogins.filter((sl) => sl.userID == this._loggedInUser!.toString())
        if (matches.length == 0) {
            console.debug("Someone forgot to unset the logged-in user when it was removed")
            this._loggedInUser = null;
            this.saveLoggedIn()
            return null;
        }
        if (matches.length != 1) {
            console.warn("Excpected to find 1 profile for active login but found", matches.length)
        }
        return matches[0] || null;
    }

    // Name of the logged-in user:
    get userName(): string|null {
        let login = this.activeLogin
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



    // "Log in" as an identity.  It gets added to the bottom if it wasn't yet present.
    logIn(newLogin: SavedLogin) {
        let foundIndex = this._savedLogins.findIndex(
            (it) => it.userID == newLogin.userID
        )
        
        if (foundIndex < 0) {
            this._savedLogins.push(newLogin)
        }

        this.writeSavedLogins()

        this._loggedInUser = UserID.fromString(newLogin.userID)
        this.saveLoggedIn()
        this.userProfileChanged()
    }

    logOut() {
        if (this._loggedInUser == null) { return }

        this._loggedInUser = null
        this.saveLoggedIn()
    }

    // Logins that the client knows about.
    get savedLogins(): readonly SavedLogin[] {
        return this._savedLogins
    }

    // Update a savedLogin in place w/ a new value.
    updateSavedLogin(login: SavedLogin) {
        let foundIndex = this._savedLogins.findIndex(
            (it) => it.userID == login.userID
        )
        if (foundIndex < 0) {
            this._savedLogins.push(login)
        } else {
            this._savedLogins[foundIndex] = login
        }

        this.writeSavedLogins()
    }

    updateSavedLogins(logins: SavedLogin[]) {
        this._savedLogins = logins
        this.writeSavedLogins()

        let currentUser = this.loggedInUser?.asBase58
        let userInLogins = logins.map(it => it.userID).filter(it => it == currentUser).length > 0
        if (!userInLogins) {
            this.logOut()
        }
    }

    getSavedLogin(userID: string): SavedLogin|null {
        return this.savedLogins.filter(it => it.userID == userID)[0] || null
    }

    // Calculate the preferred display name for a given user ID. 
    // Display names are calculated in this way:
    // * If the ID is the logged-in user, use their profile display_name.
    // * If the ID is followed by the logged-in user, and they specify a display_name, use that.
    // * If the ID has a profile that we can fetch from this server, use its display name.
    // * Otherwise, return null, this user has no preferred name.
    async getPreferredName(userID: UserID): Promise<string|null> {
        return await this.profileService.lookup(userID)
    }

    private writeSavedLogins() {
        try {
            let json = JSON.stringify(this._savedLogins)
            // TODO: Handle missing localStorage. See EditPost LocalStorageProxy example.
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

        } catch (exception) {
            console.error("Couldn't load saved logins", exception)
        }
    }

    private loadLoggedIn() {
        let userIdString = window.localStorage.getItem("logged-in-ID")
        if (userIdString === null) {
            this._loggedInUser = null
            this.userProfileChanged()
            return;
        }

        try {
            this._loggedInUser = UserID.fromString(userIdString)
            this.userProfileChanged()
        } catch (error) {
            console.error("Couldn't load logge-in user ID", error)
        }
    }

    private saveLoggedIn() {
        if (this._loggedInUser) {
            window.localStorage.setItem("logged-in-ID", this._loggedInUser.toString())
        } else {
            window.localStorage.removeItem("logged-in-ID")
        }
    }

    // Notify AppState that the currently logged-in user's profile was (possibly)
    // changed.  Allows reloading of the names cache.
    public userProfileChanged() {
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
        if (this._userID?.asBase58 == userID?.asBase58) { return }
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

    // How many seconds should we remember our private key after it's used?
    rememberKeySecs?: number

    // What was our calculated security score last time we saved those settings?
    securityScore?: number
}

// Get the proper URL to navigate to a page in the app. 
// Let us encode this in one place instead of all over the place.
export class Navigator {
    // old name: "Home"
    frontPage(): Location { return new Location(`#/home`) }
    logIn(): Location { return new Location(`#/login`) }
    newPost(userID?: UserID) { 
        if (!userID) return new Location(`#/post`) 
        return new Location(`#/u/${userID}/post`)
    }
    sync(userID?: UserID) { 
        if (!userID) return new Location(`#/sync`) 
        return new Location(`#/u/${userID}/sync`)
    }

    userFeed(user: string|UserID) {
        return new Location(`#/u/${user}/feed`)
    }

    userPosts(user: string|UserID) {
        return new Location(`#/u/${user}/`)
    }

    userProfile(user: string|UserID) {
        return new Location(`#/u/${user}/profile`)
    }

    itemPage(user: string|UserID, sig: string|Signature) {
        return new Location(`#/u/${user}/i/${sig}/`)
    }

    editProfile() {
        return new Location(`#/my_profile`)
    }

}

export class WindowScroller {
    #lastKeyboardScroll = 0
    #recentScrollThreshold = 50 // ms

    // THe user initiated keyboard navigation to scroll to this location.
    keyboardScrollTo(options: ScrollToOptions) {
        this.#lastKeyboardScroll = this.now
        window.scrollTo(options)
    }

    get scrolledViaKeyboard() {
        return this.now - this.#lastKeyboardScroll < this.#recentScrollThreshold
    }

    private get now() { return new Date().valueOf() }
}


export class Location {
    constructor(readonly hash: string) {}

    // The relative URL without the leading #
    get path() {
        return this.hash.substring(1)
    }

    goTo() {
        // TODO
    }

    // TODO: withParams(...)
}