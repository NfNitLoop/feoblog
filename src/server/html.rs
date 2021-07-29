//! Endpoints that serve plain HTML web pages.
//! 
//! These redirect any javascript-enabled browser to the client UI.
//! But any old browsers and search engines can use this to index content.

use actix_web::{HttpRequest, HttpResponse, Responder, http::StatusCode, web::{Data, Path, Query}};
use askama::Template;
use protobuf::Message;

use crate::{backend::{ItemDisplayRow, ItemRow, Signature, UserID}, markdown::ToHTML, protos::Item, server::{IndexPageItem, Nav, non_standard::identicon_url, pagination::Paginator}};
use super::{AppData, Error, ProfileFollow, pagination::Pagination};

mod filters;

pub(crate) async fn file_not_found(msg: impl Into<String>) -> impl Responder<Error=actix_web::error::Error> {
    NotFoundPage {
        message: msg.into()
    }
        .with_status(StatusCode::NOT_FOUND)
}

/// The root (`/`) page.
pub(crate) async fn view_homepage(
    data: Data<AppData>,
    Query(pagination): Query<Pagination>,
) -> Result<impl Responder, Error> {

    let mut paginator = Paginator::new(
        pagination,
        |row: ItemDisplayRow| -> Result<IndexPageItem, anyhow::Error> {        
            let mut item = Item::new();
            item.merge_from_bytes(&row.item.item_bytes)?;
            Ok(IndexPageItem{row, item})
        },
        |ipi: &IndexPageItem| -> bool {
            display_by_default(&ipi.item)
        }
    );
    paginator.max_items = 20;

    let backend = data.backend_factory.open()?;
    backend.homepage_items(paginator.time_span(), &mut paginator.callback())?;
    

    let mut nav = vec![
        Nav::Text("FeoBlog".into()),
        Nav::Link{
            text: "Client".into(),
            href: "/client/".into(),
        }
    ];

    if let Some(href) = paginator.newer_items_link("/") {
        nav.push(Nav::Link{
            text: "Newer Posts".into(),
            href,
        });
    }
    if let Some(href) = paginator.more_items_link("/") {
        nav.push(Nav::Link{
            text: "Older Posts".into(),
            href,
        });
    }

    Ok(IndexPage {
        nav,
        display_message:  paginator.message(),
        items: paginator.into_items(),
        show_authors: true,
    })
}

pub(crate) async fn get_user_feed(
    data: Data<AppData>,
    Path((user_id,)): Path<(UserID,)>,
    Query(pagination): Query<Pagination>,
) -> Result<impl Responder, Error> {
    let mut paginator = Paginator::new(
        pagination,
        |row: ItemDisplayRow| -> Result<IndexPageItem,anyhow::Error> {
            let mut item = Item::new();
            item.merge_from_bytes(&row.item.item_bytes)?;
            Ok(IndexPageItem{row, item})
        }, 
        |page_item: &IndexPageItem| { 
            display_by_default(&page_item.item)
        }
    );

    let backend = data.backend_factory.open()?;
    backend.user_feed_items(&user_id, paginator.time_span(), &mut paginator.callback())?;

    let display_name = backend.user_profile(&user_id)?.map(
        |row| -> Result<Item, anyhow::Error> {
            let mut item = Item::new();
            item.merge_from_bytes(&row.item_bytes)?;
            Ok(item)
        })
        .transpose()?
        .map(|item| -> Option<String> {
            let name = item.get_profile().display_name.trim();
            if name.len() > 0 {
                Some(name.to_string())
            } else {
                None
            }
        })
        .flatten()
        .unwrap_or_else(|| user_id.to_base58().to_string());

    let mut nav = vec![
        Nav::Text(format!("Feed for: {}", display_name)),
    ];

    nav.push(Nav::Link{text: "Profile".into(), href: "../profile/".into()});


    let this_page = format!("/u/{}/feed/", user_id.to_base58());
    if let Some(href) = paginator.newer_items_link(&this_page) {
        nav.push(Nav::Link{href, text: "Newer Posts".into()})
    };
    if let Some(href) = paginator.more_items_link(&this_page) {
        nav.push(Nav::Link{href, text: "Older Posts".into()})
    };


    Ok(IndexPage {
        nav,
        display_message: paginator.message(),
        items: paginator.into_items(),
        show_authors: true,
    })
}



/// Display a single user's posts/etc.
/// `/u/{userID}/`
pub(crate) async fn get_user_items(
    data: Data<AppData>,
    path: Path<(UserID,)>,
    Query(pagination): Query<Pagination>,
) -> Result<impl Responder, Error> {

    let mut paginator = Paginator::new(
        pagination,
        |row: ItemRow| -> Result<IndexPageItem, anyhow::Error> {
            let mut item = Item::new();
            item.merge_from_bytes(&row.item_bytes)?;
            Ok(IndexPageItem{ 
                row: ItemDisplayRow{
                    item: row,
                    // We don't display the user's name on their own page.
                    display_name: None,
                },
                item 
            })
        },
        |ipi: &IndexPageItem| -> bool {
            display_by_default(&ipi.item)
        }
    );
    paginator.max_items = 10;

    let (user,) = path.into_inner();
    let backend = data.backend_factory.open()?;
    backend.user_items(&user, paginator.time_span(), &mut paginator.callback())?;

    
    let mut nav = vec![];
    let profile = backend.user_profile(&user)?;
    if let Some(row) = profile {
        let mut item = Item::new();
        item.merge_from_bytes(&row.item_bytes)?;

        nav.push(
            Nav::Text(item.get_profile().display_name.clone())
        )
    }

    let this_url = format!("/u/{}/", user.to_base58());
    if let Some(href) = paginator.newer_items_link(&this_url) {
        nav.push(Nav::Link{ text: "Newer Posts".into(), href });
    }

    if let Some(href) = paginator.more_items_link(&this_url) {
        nav.push(Nav::Link{ text: "Older Posts".into(), href });
    }

    nav.extend(vec![
        Nav::Link{
            text: "Profile".into(),
            href: format!("/u/{}/profile/", user.to_base58()),
        },
        Nav::Link{
            text: "Feed".into(),
            href: format!("/u/{}/feed/", user.to_base58()),
        },
        Nav::Link{
            text: "Home".into(),
            href: "/".into()
        },
    ]);


    Ok(IndexPage{
        nav,
        display_message: paginator.message(),
        items: paginator.into_items(),
        show_authors: false,
    })
}


pub(crate) async fn show_item(
    data: Data<AppData>,
    path: Path<(UserID, Signature,)>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {

    let (user_id, signature) = path.into_inner();
    let backend = data.backend_factory.open()?;
    let row = backend.user_item(&user_id, &signature)?;
    let row = match row {
        Some(row) => row,
        None => { 
            // TODO: We could display a nicer error page here, showing where
            // the user might find this item on other servers. Maybe I'll leave that
            // for the in-browser client.

            return Ok(
                file_not_found("No such item").await
                .respond_to(&req).await?
            );
        }
    };

    let mut item = Item::new();
    item.merge_from_bytes(row.item_bytes.as_slice())?;

    let row = backend.user_profile(&user_id)?;
    let display_name = {
        let mut item = Item::new();
        if let Some(row) = row {
            item.merge_from_bytes(row.item_bytes.as_slice())?;
        }
        item
    }.get_profile().display_name.clone();
    
    use crate::protos::Item_oneof_item_type as ItemType;
    match item.item_type {
        None => Ok(HttpResponse::InternalServerError().body("No known item type provided.")),
        Some(ItemType::profile(_)) => Ok(HttpResponse::Ok().body("Profile update.")),
        Some(ItemType::post(p)) => {
            let page = PostPage {
                nav: vec![
                    Nav::Text(display_name.clone()),
                    Nav::Link {
                        text: "Profile".into(),
                        href: format!("/u/{}/profile/", user_id.to_base58()),
                    },
                    Nav::Link {
                        text: "Home".into(),
                        href: "/".into()
                    }
                ],
                meta: get_post_meta(&req, &user_id, &p),
                user_id,
                display_name,
                signature,
                text: p.body,
                title: p.title,
                timestamp_utc_ms: item.timestamp_ms_utc,
                utc_offset_minutes: item.utc_offset_minutes,
            };

            Ok(page.respond_to(&req).await?)
        },
        Some(ItemType::comment(_)) => Ok(
            HttpResponse::Ok().body("To view comments, please use the web client at /client/.")
        )
    }
}

fn get_post_meta(req: &HttpRequest, user_id: &UserID, post: &crate::protos::Post) -> OGPMeta {

    let info = req.connection_info();
    let scheme = info.scheme();
    let host = info.host(); // seems to include :port.

    let post_url = format!("{}://{}{}", scheme, host, req.uri().path());

    // TODO: Const somewhere?
    // We only include images that are directly attached.
    // Why? Apple Messages, for example, doesn't generate a card if the image isn't local.
    let files_prefix = "files/";

    let mut images: Vec<_> = post.body.md_get_images()
        .into_iter()
        .filter(|i| i.url.starts_with(files_prefix))
        .map(|i| OGPImage{
            url: format!("{}{}", post_url, i.url),
            alt: i.alt
        })
        .collect();

    if images.is_empty() {
        // TODO: Eventually: Show the user's profile photo, if they have one.
        // Fall back to the user's identicon:
        images.push(OGPImage{
            url: format!("{}://{}{}", scheme, host, identicon_url(user_id)),
            alt: None,
        })
    }

    OGPMeta {
        url: post_url,
        images,
        description: Some(format!("TODO: Put a description here.")),
    }
}


#[derive(Template)]
#[template(path = "index.html")] 
struct IndexPage {
    nav: Vec<Nav>,
    items: Vec<IndexPageItem>,

    /// An error/warning message to display. (ex: no items)
    display_message: Option<String>,

    /// Should we show author info w/ links to their profiles?
    show_authors: bool,
}

/// Should this Item be displayed on the plain-HTML version of the site?
/// i.e.: should it be indexed by search engines?
// TODO: Rename.
fn display_by_default(item: &Item) -> bool {
    let item_type = match &item.item_type {
        // Don't display items we can't find a type for. (newer than this server knows about):
        None => return false,
        Some(t) => t,
    };

    use crate::protos::Item_oneof_item_type as ItemType;
    match item_type {
        ItemType::post(_) => true,
        ItemType::profile(_) => false,
        ItemType::comment(_) => false,
    }
}


/// `/u/{userID}/profile/`
pub(crate) async fn show_profile(
    data: Data<AppData>,
    path: Path<(UserID,)>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> 
{
    let (user_id,) = path.into_inner();
    let backend = data.backend_factory.open()?;

    let row = backend.user_profile(&user_id)?;

    let row = match row {
        Some(r) => r,
        None => {
            return Ok(HttpResponse::NotFound().body("No such user, or profile."))
        }
    };

    let mut item = Item::new();
    item.merge_from_bytes(&row.item_bytes)?;
    let display_name = item.get_profile().display_name.clone();
    let nav = vec![
        Nav::Text(display_name.clone()),
        Nav::Link{
            text: "Posts".into(),
            href: "..".into(),
        },
        Nav::Link{
            text: "Feed".into(),
            href: "../feed/".into(),
        },
        Nav::Link{
            text: "Home".into(),
            href: "/".into(),
        },
    ];

    let timestamp_utc_ms = item.timestamp_ms_utc;
    let utc_offset_minutes = item.utc_offset_minutes;
    let text = std::mem::take(&mut item.mut_profile().about);

    let follows = std::mem::take(&mut item.get_profile()).follows.to_vec();
    let follows = follows.into_iter().map(|mut follow: crate::protos::Follow | -> Result<ProfileFollow, Error>{
        let mut user = std::mem::take(follow.mut_user());
        let user_id = UserID::from_vec(std::mem::take(&mut user.bytes))?;
        let display_name = follow.display_name;
        Ok(
            ProfileFollow{user_id, display_name}
        )
    }).collect::<Result<_,_>>()?;

    let page = ProfilePage{
        nav,
        text,
        display_name,
        follows,
        timestamp_utc_ms,
        utc_offset_minutes,
        user_id: row.user,
        signature: row.signature,
    };

    Ok(page.respond_to(&req).await?)
}



#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFoundPage {
    message: String,
}

#[derive(Template)]
#[template(path = "profile.html")]
struct ProfilePage {
    nav: Vec<Nav>,
    user_id: UserID,
    signature: Signature,
    display_name: String,
    text: String,
    follows: Vec<ProfileFollow>,
    timestamp_utc_ms: i64,
    utc_offset_minutes: i32,
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostPage {
    nav: Vec<Nav>,
    user_id: UserID,
    signature: Signature,
    display_name: String,
    text: String,
    title: String,
    timestamp_utc_ms: i64,
    utc_offset_minutes: i32,

    meta: OGPMeta,
}

/// Open Graph Protocol Metadata
///
/// See: https://ogp.me/
struct OGPMeta {
    // The FQ URL of this item.
    // make sure to detect hostname/port from request. 
    url: String,

    // TODO: Try to parse this out of the Markdown?
    description: Option<String>,

    // Already included in PostPage: title, timestamp
    // Should fall back to some other image 
    images: Vec<OGPImage>,

}

struct OGPImage {
    url: String,

    /// alt text. (NOT a caption, says ogp.me)
    alt: Option<String>
}