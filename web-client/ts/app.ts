import type { Item, Profile } from "../protos/feoblog"
import type { UserID } from "./client"

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
    private _userID: UserID|null = null
    private _userProfile: Item|null = null

    constructor() {
        // TODO: Check browser store to see if we're logged in as a particular user.
    }

    get loggedIn(): boolean {
        return this.loggedInUser != null
    }

    get loggedInUser(): UserID|null {
        return this._userID
    }

    get loggedInProfile(): Profile|null {
        return this._userProfile?.profile || null
    }

    // Name of the logged-in user:
    get userName(): String|null {
        return this.loggedInProfile?.display_name || null
    }

    login(userID: UserID, profile: Item|null) {
        this._userID = userID
        this._userProfile = profile
    }

    logout() {
        this._userID = null
        this._userProfile = null
    }

}