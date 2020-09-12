use futures::Stream;
use std::fmt;

use actix_web::http::header;
use actix_web::web::{
    self,
    get,
    put,
    resource,
    route,
    Data,
    Form,
    HttpResponse,
    Path,
    HttpRequest,
    Payload,
};
use actix_web::{App, HttpServer, Responder};
use askama::Template;
use failure::{bail, ResultExt};
use serde::Deserialize;
use rust_embed::RustEmbed;

use crate::backend::{self, Backend, Factory, UserID, Signature, Hash};
use crate::responder_util::ToResponder;
use actix_web::http::StatusCode;
use async_trait::async_trait;


pub(crate) fn serve(options: crate::SharedOptions) -> Result<(), failure::Error> {

    // TODO: Error if the file doesn't exist, and make a separate 'init' command.
    let factory = backend::sqlite::Factory::new(options.sqlite_file.clone());
    // For now, this creates one if it doesn't exist already:
    factory.open()?.setup().context("Error setting up DB")?;
    

    let app_factory = move || {
        let db = factory.open().expect("Couldn't open DB connection.");
        let mut app = App::new()
            .data(db)
            .configure(routes)
        ;

        app = app.default_service(route().to(file_not_found));

        return app;
    };

    let server = HttpServer::new(app_factory).bind("127.0.0.1:8080")?;
    let url = "http://127.0.0.1:8080/";
    // TODO: This opens up a (AFAICT) blocking CLI browser on Linux. Boo. Don't do that.
    // TODO: Also move this outside of the server module.
    let opened = webbrowser::open(url);
    if !opened.is_ok() {
        println!("Warning: Couldn't open browser.");
    }
    println!("Started at: {}", url);

    // TODO: Pass this to an async runner.
    let mut system = actix_web::rt::System::new("web server");
    system.block_on(server.run())?;
   
    Ok(())
}

/// Routes appropriate for servers and local use.
fn routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/", get().to(index))

        // .route("/u/{userID}/i/{signature}/", get().to(TODO))
        .route("/u/{user_id}/i/{signature}/proto3", put().to(put_item))
        

        // TODO: view raw markdown
        // .route("/md/{base58hash}", get().to(view_md))
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
            .body("File not found.")
        )
    }
} 


#[derive(RustEmbed, Debug)]
#[folder = "static/"]
struct StaticFiles;

#[derive(RustEmbed, Debug)]
#[folder = "web-client/build/"]
struct WebClientBuild;




fn statics(cfg: &mut web::ServiceConfig) {
    cfg
        // .route(
        //     "/style.css",
        //     get().to(|| {
        //         HttpResponse::Ok()
        //             .body(include_str!("../static/style.css"))
        //            s.with_header("content-type", "text/css")
        //     })
        // )
        .route("/static/{path:.*}", get().to(StaticFiles::response))
        // .route("/web-cli/modules/{path:.*}", get().to(WebClientDeps::response))
        // .route("/web-cli/dist/{path:.*}", get().to(WebClientDist::response))
        .route("/client/{path:.*}", get().to(WebClientBuild::response))
    ;
}

async fn index(backend: Data<Box<dyn Backend>>) -> Result<impl Responder, Error> {
    // TODO: Update this to show homepage posts.

    let response = IndexPage {
        name: "World".into(),
        hashes: vec![],
    }
    .responder();

    Ok(response)
}

const MAX_ITEM_SIZE: usize = 1024 * 32; 

/// Accepts a proto3 Item
/// Returns 200 if the PUT was successful.
/// Returns ??? if the item already exists.
/// Returns ??? if the user lacks permission to post.
/// Returns ??? if the signature is not valid.
/// Returns a text body message w/ OK/Error message.
async fn put_item(
    backend: Data<Box<dyn Backend>>,
    path: Path<(String, String,)>,
    req: HttpRequest,
    body: Payload,
) -> Result<impl Responder, Error> 
{
    let (user_path, sig_path) = path.into_inner();
    let user = UserID::from_base58(user_path.as_str()).context("decoding user ID").compat()?;
    let signature = Signature::from_base58(sig_path.as_str()).context("decoding signature").compat()?;

    let length = match req.headers().get("content-length") {
        Some(length) => length,
        None => {
            return Ok(
                HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body("Must include length header.".to_string())
            );
        }
    };

    let length: usize = match length.to_str()?.parse() {
        Ok(length) => length,
        Err(_) => {
            return Ok(
                HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body("Must include length header.".to_string())
            );
        },
    };

    if length > MAX_ITEM_SIZE {
        return Ok(
            HttpResponse::BadRequest()
            .content_type("text/plain; charset=utf-8")
            .body("Item too large".to_string())
            
        );
    }

    println!("Checkintg user");

    // TODO: Eventually also check if this user is "followed". Their content
    // can be posted here too.
    let can_post = backend.server_user(&user).compat()?.is_some();

    if !can_post {
        return Ok(
            HttpResponse::Forbidden()
            .content_type("text/plain; charset=utf-8")
            .body("Not accepting Items for this user".to_string())
        )
    }

    println!("user OK");

    // Payloads can only be fetched async in Actix? 
    // And async in Actix v1 is a PITA.

    let mut bytes: Vec<u8> = Vec::with_capacity(length);
    // while let Some(chunk) = body.next().await {
    //     println!("Got chunk.");
    //     let chunk = match chunk {
    //         Ok(chunk) => chunk,
    //         Err(err) => {
    //             bail!("{}", err.to_string());
    //         }
    //     };
    //     bytes.extend_from_slice(&chunk);
    // }

    // TODO: Check signature.
    // TODO: Parse & validate Item.
    // TODO: Save Item.
    let message = format!("OK. Got {} bytes", bytes.len());
       
    let response = HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(message);

    Ok(response)
}


// fn view_md(
//     backend: Data<Box<dyn Backend>>,
//     path: Path<(String,)>,
// ) -> Result<impl Responder, Error> {
//     let (base58hash,) = path.into_inner();
//     let hash = Hash::from_base58(base58hash.as_ref())?;
//     let result = backend.get_blob(&hash)?.unwrap_or("No result.".into());
//     let result = String::from_utf8(result)?;

//     let parser = pulldown_cmark::Parser::new(&result);
//     use pulldown_cmark::Event::*;
//     let parser = parser.map(|event| match event {
//         Html(value) => Code(value),
//         InlineHtml(value) => Text(value),
//         x => x,
//     });

//     let mut html = String::new();
//     pulldown_cmark::html::push_html(&mut html, parser);

//     let response = HttpResponse::Ok()
//         .content_type("text/html; charset=utf-8")
//         .body(html);
//     Ok(response)
// }


async fn file_not_found() -> impl Responder {
    NotFoundPage {}
        .responder()
        .with_status(StatusCode::NOT_FOUND)
}




#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFoundPage {}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage {
    name: String,
    hashes: Vec<Hash>,
}

#[derive(Template, Default)]
#[template(path = "post.html")]
struct PostPage {
}

/// A type implementing ResponseError that can hold any kind of std::error::Error.
#[derive(Debug)]
struct Error {
    inner: Box<dyn std::error::Error + 'static>
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> { 
        self.inner.fmt(formatter)
    }
}

impl actix_web::error::ResponseError for Error {}

impl <E> From<E> for Error
where E: std::error::Error + 'static
{
    fn from(err: E) -> Self {
        Error{
            inner: err.into()
        }
    }
}