//! Non-standard endpoints.
//!
//! These are not part of the documented FeoBlog standard, but are used by this
//! particular FeoBlog implementation to provide extra features.

use actix_web::{HttpResponse, web::Path};
use identicon::IdenticonJSOptions;

use crate::backend::UserID;

/// This is not really defined as part of the standard for FeoBlogs.
/// BUT, having a default user image is handy when implementing the Open Graph Protocol.
/// (... which is itself also not a strict requirement for a FeoBlog.)
pub(crate) fn identicon_get(Path(user_id): Path<UserID>) -> HttpResponse {
    use identicon::{Identicon, Mode::IdenticonJS};

    // Note: Must be >=16 bytes, but userIDs are bigger:
    let icon = Identicon::new(user_id.bytes())
        .mode(IdenticonJS(Default::default()))
    ;

    let mut png = vec![];    
    if let Err(err) = icon.to_png(&mut png) {
        return HttpResponse::InternalServerError()
            .body(format!("Error: {}", err));
    }

    HttpResponse::Ok()
        .content_type("image/png")
        .body(png)
}

/// server-absolute identicon URL. ex:  /u/__/icon.png
pub(crate) fn identicon_url(user_id: &UserID) -> String {
    format!("/u/{}/icon.png", user_id.to_base58())
}
