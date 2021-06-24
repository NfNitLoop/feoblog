use std::{borrow::Cow, fmt, fmt::Write, marker::PhantomData, net::TcpListener, ops::{Deref, DerefMut}};

// TODO: This module is getting long.
// Split it out into parts:
// * Parts that render static HTML pages
// * Parts that return/accept Protobuf3 data required for clients.
// * Static file handling logic.
// * etc?

use backend::FactoryBox;
use futures::{Future, StreamExt};

use actix_web::{dev::{HttpResponseBuilder, Service, ServiceRequest, ServiceResponse}, http::{Method, header::ContentType}, middleware::DefaultHeaders, web::Query};
use actix_web::web::{
    self,
    get,
    put,
    route,
    Data,
    HttpResponse,
    Path,
    HttpRequest,
    Payload,
};
use actix_web::{App, HttpServer, Responder};
use askama::Template;
use anyhow::{Context, format_err};
use log::debug;
use logging_timer::timer;
use rust_embed::RustEmbed;
use serde::Deserialize;

use actix_web::http::StatusCode;
use async_trait::async_trait;

use protobuf::Message;

use crate::{ServeCommand, backend::{ItemDisplayRow, TimeSpan}, protos::{ItemList, ItemListEntry, ItemType, Item_oneof_item_type}};
use crate::backend::{self, UserID, Signature, ItemRow, Timestamp};
use crate::protos::{Item, ProtoValid};

mod attachments;
mod filters;
mod pagination;

use pagination::Paginator;

pub(crate) fn serve(command: ServeCommand) -> Result<(), anyhow::Error> {

    env_logger::init();
    sodiumoxide::init().expect("sodiumoxide::init()");

    let ServeCommand{open, backend_options, mut binds} = command;

    let factory_box = FactoryBox{
        factory: backend_options.factory_builder()?.factory()?
    };

    let app_factory = move || {
        let mut app = App::new()
            .wrap(actix_web::middleware::Logger::default())
            .data(AppData{
                backend_factory: factory_box.factory.dyn_clone(),
            })
            .configure(routes)
        ;

        app = app.default_service(route().to(|| file_not_found("")));

        return app;
    };

    if binds.is_empty() {
        binds.push("127.0.0.1:8080".into());
    }

    let mut server = HttpServer::new(app_factory); 
    
    for bind in &binds {
        let socket = open_socket(bind).with_context(|| {
            format!("Error binding to address/port: {}", bind)
        })?;
        server = server.listen(socket)?;
    }

    if open {
        // TODO: This opens up a (AFAICT) blocking CLI browser on Linux. Boo. Don't do that.
        // TODO: Handle wildcard addresses (0.0.0.0, ::0) and --open them via localhost.
        let url = format!("http://{}/", binds[0]);
        let opened = webbrowser::open(&url);
        if !opened.is_ok() {
            println!("Warning: Couldn't open browser.");
        }
    }

    for bind in &binds {
        println!("Started at: http://{}/", bind);
    }
 
    let mut system = actix_web::rt::System::new("web server");
    system.block_on(server.run())?;
   
    Ok(())
}

// Work around https://github.com/actix/actix-web/issues/1913
fn open_socket(bind: &str) -> Result<TcpListener, anyhow::Error> {
    use socket2::{Domain, Protocol, Socket, Type};
    use std::net::SocketAddr;
    
    // Eh, this is what actix was using:
    let backlog = 1024;
    
    let addr = bind.parse()?;
    let domain = match addr {
        SocketAddr::V4(_) => Domain::ipv4(),
        SocketAddr::V6(_) => Domain::ipv6(),
    };
    let socket = Socket::new(domain, Type::stream(), Some(Protocol::tcp()))?;
    socket.bind(&addr.into())?;
    socket.listen(backlog)?;

    Ok(socket.into_tcp_listener())
}

/// Data available for our whole application.
/// Gets stored in a Data<AppData>
// This is so that we have typesafe access to AppData fields, because actix
// Data<Foo> can fail at runtime if you delete a Foo and don't clean up after
// yourself.
pub(crate) struct AppData {
    backend_factory: Box<dyn backend::Factory>,
}

fn routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/", get().to(view_homepage))
        .route("/homepage/proto3", get().to(homepage_item_list))

        .route("/u/{user_id}/", get().to(get_user_items))
        .service(
            web::resource("/u/{user_id}/proto3")
            .route(get().to(user_item_list))
            .wrap(cors_ok_headers())
        )


        .route("/u/{userID}/i/{signature}/", get().to(show_item))
        .service(
            web::resource("/u/{userID}/i/{signature}/proto3")
            .route(get().to(get_item))
            .route(put().to(put_item))
            .route(route().method(Method::OPTIONS).to(cors_preflight_allow))
            .wrap(cors_ok_headers())
            .wrap_fn(immutable_etag)
        )
        .service(
            web::resource("/u/{user_id}/i/{signature}/replies/proto3")
            .route(get().to(item_reply_list))
            .wrap(cors_ok_headers())
        ).service(
            web::resource("/u/{user_id}/i/{signature}/files/{file_name}")
            .route(get().to(attachments::get_file))
            .route(put().to(attachments::put_file))
            .route(route().method(Method::HEAD).to(attachments::head_file))
            .route(route().method(Method::OPTIONS).to(cors_preflight_allow))
            .wrap(cors_ok_headers())
            .wrap_fn(immutable_etag)
        )

        .route("/u/{user_id}/profile/", get().to(show_profile))
        .service(
            web::resource("/u/{user_id}/profile/proto3")
            .route(get().to(get_profile_item))
            .wrap(cors_ok_headers())
        )
        .route("/u/{user_id}/feed/", get().to(get_user_feed))
        .route("/u/{user_id}/feed/proto3", get().to(feed_item_list))

    ;
    statics(cfg);
}

#[async_trait]
trait StaticFilesResponder {
    type Response: Responder;
    async fn response(path: Path<(String,)>) -> Result<Self::Response, Error>;
}

#[async_trait]
impl <T: RustEmbed> StaticFilesResponder for T {
    type Response = HttpResponse;

    async fn response(path: Path<(String,)>) -> Result<Self::Response, Error> {
        let (mut path,) = path.into_inner();
        
            
        let mut maybe_bytes = T::get(path.as_str());
        
        // Check index.html:
        if maybe_bytes.is_none() && (path.ends_with("/") || path.is_empty()) {
            let inner = format!("{}index.html", path);
            let mb = T::get(inner.as_str());
            if mb.is_some() {
                path = inner;
                maybe_bytes = mb;
            }
        }

        if let Some(bytes) = maybe_bytes {
            // Set some response headers.
            // In particular, a mime type is required for things like JS to work.
            let mime_type = format!("{}", mime_guess::from_path(path).first_or_octet_stream());
            let response = HttpResponse::Ok()
                .content_type(mime_type)

                // TODO: This likely will result in lots of byte copying.
                // Should implement our own MessageBody
                // for Cow<'static, [u8]>
                .body(bytes.into_owned());
            return Ok(response)
        }

        // If adding the slash would get us an index.html, do so:
        let with_index = format!("{}/index.html", path);
        if T::get(with_index.as_str()).is_some() {
            // Use a relative redirect from the inner-most path part:
            let part = path.split("/").last().expect("at least one element");
            let part = format!("{}/", part);
            return Ok(
                HttpResponse::SeeOther()
                    .header("location", part)
                    .finish()
            );
        }

        Ok(
            HttpResponse::NotFound()
            .set(ContentType::plaintext())
            .body("File not found.")
        )
    }
}

/// Browsers like to re-validate things even when they don't need to. (Say, when the user hits reload.)
/// For our content-addressable URLs, make a shortcut etag to spare us some bandwidth & DB hits:
fn immutable_etag<'a, S>(req: ServiceRequest, service: &'a mut S) 
-> impl Future<Output = Result<ServiceResponse, actix_web::error::Error>>
where S: Service<Request=ServiceRequest, Response=ServiceResponse, Error=actix_web::error::Error>
{
    use actix_web::Either;
    use actix_web::http::header::{self, HeaderName, HeaderValue};

    let is_get = req.method() == &Method::GET;
    // If the client sends us an if-none-match, they're just sending back our "immutable" ETag.
    // This means they already have our data and are just trying to re-load it unnecessarily.
    let cache_validation_request = req.headers().get("if-none-match").is_some();


    let fut = if !cache_validation_request {
        Either::A(service.call(req))
    } else {
        // Skip dispatching to the underlying service, and pass along the req:
        Either::B(req)
    };
    async move {
        let mut res = match fut {
            Either::A(fut) => fut.await?,
            Either::B(req) => {
                let res = HttpResponse::NotModified().body("");
                let res = req.into_response(res);
                return Ok(res);
            }
        };

        if is_get && res.response().status().is_success() {
            let headers = res.headers_mut();
            headers.insert(header::ETAG, HeaderValue::from_static("\"immutable\""));
                    
            // "aggressive caching" according to https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control
            // 31536000 = 365 days, as seconds
            headers.insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=31536000, no-transform, immutable")
            );
        }

        Ok(res)
    }
}

// Note: This function signature DID NOT WORK with wrap_fn(), and produced
// confusing error messages. If anyone can clarify to me why, I'd be very happy
// to know.
// See: https://twitter.com/NfNitLoop/status/1361389613672062978
//
// async fn immutable_etag<S>(req: ServiceRequest, service: S) 
// -> Result<ServiceResponse, actix_web::error::Error> 
// where for<'a> &'a mut S: Service
// {
//     todo!()
// }


#[derive(RustEmbed, Debug)]
#[folder = "static/"]
struct StaticFiles;

#[derive(RustEmbed, Debug)]
#[folder = "web-client/build/"]
struct WebClientBuild;


fn statics(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/static/{path:.*}", get().to(StaticFiles::response))
        .route("/client/{path:.*}", get().to(WebClientBuild::response))
    ;
}

/// Set lower and upper bounds for input T.
fn bound<T: Ord>(input: T, lower: T, upper: T) -> T {
    use std::cmp::{min, max};
    min(max(lower, input), upper)
}


/// The root (`/`) page.
async fn view_homepage(
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

fn item_to_entry(item: &Item, user_id: &UserID, signature: &Signature) -> ItemListEntry {
    let mut entry = ItemListEntry::new();
    entry.set_timestamp_ms_utc(item.timestamp_ms_utc);
    entry.set_signature({
        let mut sig = crate::protos::Signature::new();
        sig.set_bytes(signature.bytes().into());
        sig
    });
    entry.set_user_id({
        let mut uid = crate::protos::UserID::new();
        uid.set_bytes(user_id.bytes().into());
        uid
    });
    entry.set_item_type(
        match item.item_type {
            Some(Item_oneof_item_type::post(_)) => ItemType::POST,
            Some(Item_oneof_item_type::profile(_)) => ItemType::PROFILE,
            Some(Item_oneof_item_type::comment(_)) => ItemType::COMMENT,
            None => ItemType::UNKNOWN,
        }
    );

    entry
}

// Get the protobuf ItemList for items on the homepage.
async fn homepage_item_list(
    data: Data<AppData>,
    Query(pagination): Query<Pagination>,
) -> Result<HttpResponse, Error> {

    let mut paginator = Paginator::new(
        pagination,
        |row: ItemDisplayRow| -> Result<ItemListEntry,anyhow::Error> {
            let mut item = Item::new();
            item.merge_from_bytes(&row.item.item_bytes)?;
            Ok(item_to_entry(&item, &row.item.user, &row.item.signature))
        }, 
        |entry: &ItemListEntry| { 
            entry.get_item_type() == ItemType::POST
        }
    );
    // We're only holding ItemListEntries in memory, so we can up this limit and save some round trips.
    paginator.max_items = 1000;

    let backend = data.backend_factory.open()?;
    backend.homepage_items(paginator.time_span(), &mut paginator.callback())?;
    

    let mut list = ItemList::new();
    list.no_more_items = !paginator.has_more;
    list.items = protobuf::RepeatedField::from(paginator.into_items());
    Ok(
        proto_ok().body(list.write_to_bytes()?)
    )
}

// Start building a response w/ proto3 binary data.
fn proto_ok() -> HttpResponseBuilder {
    let mut builder = HttpResponse::Ok();
    builder.content_type("application/protobuf3");
    builder
}

// // CORS headers must be present for *all* responses, including 404, 500, etc.
// // Applying it to each case individiaully may be error-prone, so here's a filter to do so for us.

fn cors_ok_headers() -> DefaultHeaders {
    DefaultHeaders::new()
    .header("Access-Control-Allow-Origin", "*")
    .header("Access-Control-Expose-Headers", "*")

    // Number of seconds a browser can cache the cors allows.
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Max-Age
    // FF caps this at 24 hours, and is the most permissive there, so that's what we'll use.
    // Does this mean that my Cache-Control max-age is truncated to this value? That would be sad.
    .header("Access-Control-Max-Age", "86400")
}

// Before browsers will post data to a server, they make a CORS OPTIONS request to see if that's OK.
// This responds to that request to let the client know this request is allowed.
async fn cors_preflight_allow() -> HttpResponse {
    HttpResponse::NoContent()
        .header("Access-Control-Allow-Methods", "OPTIONS, GET, PUT, HEAD")
        .body("")
}

async fn feed_item_list(
    data: Data<AppData>,
    Path((user_id,)): Path<(UserID,)>,
    Query(pagination): Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let mut paginator = Paginator::new(
        pagination,
        |row: ItemDisplayRow| -> Result<ItemListEntry,anyhow::Error> {
            let mut item = Item::new();
            item.merge_from_bytes(&row.item.item_bytes)?;
            Ok(item_to_entry(&item, &row.item.user, &row.item.signature))
        }, 
        |_: &ItemListEntry| { true } // include all items
    );
    // We're only holding ItemListEntries in memory, so we can up this limit and
    // save some round trips.
    paginator.max_items = 1000;

    let backend = data.backend_factory.open()?;

    // Note: user_feed_items is doing a little bit of extra work to fetch
    // display_name, which we then throw away. We *could* make a more efficient
    // version that we use for just this case, but eh, reuse is nice.
    backend.user_feed_items(&user_id, paginator.time_span(), &mut paginator.callback())?;

    let mut list = ItemList::new();
    list.no_more_items = !paginator.has_more;
    list.items = protobuf::RepeatedField::from(paginator.into_items());
    Ok(
        proto_ok()
        .body(list.write_to_bytes()?)
    )
}

async fn user_item_list(
    data: Data<AppData>,
    Path((user_id,)): Path<(UserID,)>,
    Query(pagination): Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let mut paginator = Paginator::new(
        pagination,
        |row: ItemRow| -> Result<ItemListEntry,anyhow::Error> {
            let mut item = Item::new();
            item.merge_from_bytes(&row.item_bytes)?;
            Ok(item_to_entry(&item, &row.user, &row.signature))
        }, 
        |_| { true } // include all items
    );
    // We're only holding ItemListEntries in memory, so we can up this limit and
    // save some round trips.
    paginator.max_items = 1000;

    let backend = data.backend_factory.open()?;

    // Note: user_feed_items is doing a little bit of extra work to fetch
    // display_name, which we then throw away. We *could* make a more efficient
    // version that we use for just this case, but eh, reuse is nice.
    backend.user_items(&user_id, paginator.time_span(), &mut paginator.callback())?;

    let mut list = ItemList::new();
    list.no_more_items = !paginator.has_more;
    list.items = protobuf::RepeatedField::from(paginator.into_items());
    Ok(
        proto_ok()
        .body(list.write_to_bytes()?)
    )
}

async fn item_reply_list(
    data: Data<AppData>,
    Path((user_id, signature)): Path<(UserID, Signature)>,
    Query(pagination): Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let mut paginator = Paginator::new(
        pagination,
        |row: ItemRow| -> Result<ItemListEntry,anyhow::Error> {
            let mut item = Item::new();
            item.merge_from_bytes(&row.item_bytes)?;
            Ok(item_to_entry(&item, &row.user, &row.signature))
        }, 
        |_| { true } // include all items
    );
    // We're only holding ItemListEntries in memory, so we can up this limit and
    // save some round trips.
    paginator.max_items = 1000;

    let backend = data.backend_factory.open()?;

    // Note: user_feed_items is doing a little bit of extra work to fetch
    // display_name, which we then throw away. We *could* make a more efficient
    // version that we use for just this case, but eh, reuse is nice.
    backend.reply_items(&user_id, &signature, paginator.before(), &mut paginator.callback())?;

    let mut list = ItemList::new();
    list.no_more_items = !paginator.has_more;
    list.items = protobuf::RepeatedField::from(paginator.into_items());
    Ok(
        proto_ok()
        .body(list.write_to_bytes()?)
    )
}

#[derive(Deserialize, Debug)]
pub(crate) struct Pagination {
    /// Time before which to show posts. Default is now.
    before: Option<i64>,

    /// Time after which to show some posts. can not set before & after, and before takes precedence.
    after: Option<i64>,

    /// Limit how many posts appear on a page.
    count: Option<usize>,
}



async fn get_user_feed(
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
async fn get_user_items(
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

const MAX_ITEM_SIZE: usize = 1024 * 32; 
const PLAINTEXT: &'static str = "text/plain; charset=utf-8";

/// Accepts a proto3 Item
/// Returns 201 if the PUT was successful.
/// Returns 202 if the item already exists.
/// Returns ??? if the user lacks permission to post.
/// Returns ??? if the signature is not valid.
/// Returns a text body message w/ OK/Error message.
async fn put_item(
    data: Data<AppData>,
    path: Path<(String, String,)>,
    req: HttpRequest,
    mut body: Payload,
) -> Result<HttpResponse, Error> 
{
    let _timer = timer!("put_item()");

    let (user_path, sig_path) = path.into_inner();
    let user = UserID::from_base58(user_path.as_str()).context("decoding user ID")?;
    let signature = Signature::from_base58(sig_path.as_str()).context("decoding signature")?;

    let length = match req.headers().get("content-length") {
        Some(length) => length,
        None => {
            return Ok(
                HttpResponse::LengthRequired()
                .content_type(PLAINTEXT)
                .body("Must include length header.".to_string())
                // ... so that we can reject things that are too large outright.
            );
        }
    };

    let length: usize = match length.to_str()?.parse() {
        Ok(length) => length,
        Err(_) => {
            return Ok(
                HttpResponse::BadRequest()
                .content_type(PLAINTEXT)
                .body("Error parsing Length header.".to_string())
            );
        },
    };

    if length > MAX_ITEM_SIZE {
        return Ok(
            HttpResponse::PayloadTooLarge()
            .content_type(PLAINTEXT)
            .body(format!("Item must be <= {} bytes", MAX_ITEM_SIZE))
        );
    }

    let mut backend = data.backend_factory.open()?;

    // If the content already exists, do nothing.
    if backend.user_item_exists(&user, &signature)? {
        return Ok(
            HttpResponse::Accepted()
            .content_type(PLAINTEXT)
            .body("Item already exists")
        );
    }

    if !backend.user_known(&user)? {
        return Ok(
            HttpResponse::Forbidden()
            .content_type(PLAINTEXT)
            .body("Unknown user ID")
        )
    }
    
    let mut bytes: Vec<u8> = Vec::with_capacity(length);
    while let Some(chunk) = body.next().await {
        let chunk = chunk.context("Error parsing chunk")?;
        bytes.extend_from_slice(&chunk);
    }

    if !signature.is_valid(&user, &bytes) {
        Err(format_err!("Invalid signature"))?;
    }

    let mut item: Item = Item::new();
    item.merge_from_bytes(&bytes)?;
    item.validate()?;

    if item.timestamp_ms_utc > Timestamp::now().unix_utc_ms {
        return Ok(
            HttpResponse::BadRequest()
            .content_type(PLAINTEXT)
            .body("The Item's timestamp is in the future")
        )
    }

    if let Some(deny_reason) = backend.quota_check_item(&user, &bytes, &item)? {
        return Ok(
            HttpResponse::InsufficientStorage()
            .body(format!("{}", deny_reason))
        )
    }

    let message = format!("OK. Received {} bytes.", bytes.len());
    
    let row = ItemRow{
        user: user,
        signature: signature,
        timestamp: Timestamp{ unix_utc_ms: item.get_timestamp_ms_utc()},
        received: Timestamp::now(),
        item_bytes: bytes,
    };

    let timer = timer!("save_user_item");
    backend.save_user_item(&row, &item).context("Error saving user item")?;
    drop(timer);

    let response = HttpResponse::Created()
        .content_type(PLAINTEXT)
        .body(message);

    Ok(response)
}


async fn show_item(
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
            HttpResponse::Ok().body("TODO: Display comments in HTML")
        )
    }


}

/// Get the binary representation of the item.
///
/// `/u/{userID}/i/{sig}/proto3`
async fn get_item(
    data: Data<AppData>,
    path: Path<(UserID, Signature,)>,
) -> Result<HttpResponse, Error> {
    let (user_id, signature) = path.into_inner();
    let backend = data.backend_factory.open()?;
    let item = backend.user_item(&user_id, &signature)?;
    let item = match item {
        Some(item) => item,
        None => { 
            return Ok(
                HttpResponse::NotFound().body("No such item")
            );
        }
    };

    // We could in theory validate the bytes ourselves, but if a client is directly fetching the 
    // protobuf bytes via this endpoint, it's probably going to be so that it can verify the bytes
    // for itself anyway.
    Ok(
        proto_ok()
        .body(item.item_bytes)
    )

}

/// Get the latest profile we have for a user ID.
/// returns the signature in a "signature" header so clients can verify it.
async fn get_profile_item(
    data: Data<AppData>,
    Path((user_id,)): Path<(UserID,)>,
) -> Result<HttpResponse, Error> {
    
    let backend = data.backend_factory.open()?;
    let item = backend.user_profile(&user_id,)?;
    let item = match item {
        Some(item) => item,
        None => { 
            return Ok(
                HttpResponse::NotFound().body("No such item")
            );
        }
    };

    // We could in theory validate the bytes ourselves, but if a client is directly fetching the 
    // protobuf bytes via this endpoint, it's probably going to be so that it can verify the bytes
    // for itself anyway.
    Ok(
        proto_ok()
        .header("signature", item.signature.to_base58())
        .body(item.item_bytes)
    )

}
async fn file_not_found(msg: impl Into<String>) -> impl Responder<Error=actix_web::error::Error> {
    NotFoundPage {
        message: msg.into()
    }
        .with_status(StatusCode::NOT_FOUND)
}

/// `/u/{userID}/profile/`
async fn show_profile(
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
#[template(path = "index.html")] 
struct IndexPage {
    nav: Vec<Nav>,
    items: Vec<IndexPageItem>,

    /// An error/warning message to display. (ex: no items)
    display_message: Option<String>,

    /// Should we show author info w/ links to their profiles?
    show_authors: bool,
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

    // TODO: Include comments from people this user follows.
}

struct ProfileFollow {
    /// May be ""
    display_name: String,
    user_id: UserID,
}

/// An Item we want to display on a page.
struct IndexPageItem {
    row: ItemDisplayRow,
    item: Item,
}

impl IndexPageItem {
    fn item(&self) -> &Item { &self.item }
    fn row(&self) -> &ItemDisplayRow { &self.row }

    fn display_name(&self) -> Cow<'_, str>{
        self.row.display_name
            .as_ref()
            .map(|n| n.trim())
            .map(|n| if n.is_empty() { None } else { Some (n) })
            .flatten()
            .map(|n| n.into())
            // TODO: Detect/protect against someone setting a userID that mimics a pubkey?
            .unwrap_or_else(|| self.row.item.user.to_base58().into())
    }
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

/// Represents an item of navigation on the page.
enum Nav {
    Text(String),
    Link{
        text: String,
        href: String,
    },
}


/// A type implementing ResponseError that can hold any kind of std::error::Error.
#[derive(Debug)]
pub(crate) struct Error {
    inner: Box<dyn std::error::Error + 'static>
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> { 
        self.inner.fmt(formatter)
    }
}

impl actix_web::error::ResponseError for Error {}

impl <E> From<E> for Error
where E: Into<Box<dyn std::error::Error + 'static>>
{
    fn from(inner: E) -> Self {
        Error{
            inner: inner.into()
        }
    }
}

/// An Error that is also Send, required in some cases:
#[derive(Debug)]
pub struct SendError {
    inner: Box<dyn std::error::Error + Send + 'static>
}

impl fmt::Display for SendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> { 
        self.inner.fmt(formatter)
    }
}

impl actix_web::error::ResponseError for SendError {}

impl <E> From<E> for SendError
where E: std::error::Error + Send + 'static
{
    fn from(err: E) -> Self {
        Self{
            inner: Box::new(err)
        }
    }
}
