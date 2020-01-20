use actix_web::http::header;
use actix_web::web::{self, get, post, resource, route, Data, Form, HttpResponse, Path};
use actix_web::{App, HttpServer, Responder};
use askama::Template;
use failure::{bail, Error, ResultExt};
use serde::Deserialize;
use rust_embed::RustEmbed;

use crate::backend::{self, *};
use crate::responder_util::ToResponder;
use actix_web::http::StatusCode;
use rust_base58::{FromBase58, ToBase58};



pub(crate) fn serve(options: crate::SharedOptions) -> Result<(), Error> {
    rust_sodium::init().expect("rust_sodium::init()");

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

        if options.allow_login {
            app = app.configure(logged_in_routes);
        }

        app = app.default_service(route().to(file_not_found));

        return app;
    };

    let server = HttpServer::new(app_factory).bind("127.0.0.1:8080")?;
    let url = "http://127.0.0.1:8080/";
    // TODO: This opens up a (AFAICT) blocking CLI browser on Linux. Boo. Don't do that.
    let opened = webbrowser::open(url);
    if !opened.is_ok() {
        println!("Warning: Couldn't open browser.");
    }
    println!("Started at: {}", url);

    server.run()?; // Actually blocks & runs forever.

    Ok(())
}

/// Routes appropriate for servers and local use.
fn routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/", get().to(index))
        .route("/blob/{base58hash}", get().to(view_blob))
        .route("/md/{base58hash}", get().to(view_md))
    ;
    statics(cfg);
}



trait StaticFilesResponder {
    type Response: Responder;
    fn response(path: Path<(String,)>) -> Result<Self::Response, Error>;
}

impl <T: RustEmbed> StaticFilesResponder for T {
    type Response = HttpResponse;

    fn response(path: Path<(String,)>) -> Result<Self::Response, Error> {
        let (path,) = path.into_inner();
        
            
        let bytes = match T::get(path.as_str()) {
            Some(value) => value,
            _ => return Ok(
                    HttpResponse::NotFound()
                    .body("File not found.")
                )
        };

        // Set some response headers.
        // In particular, a mime type is required for things like JS to work.
        let mime_type = format!("{}", mime_guess::from_path(path).first_or_octet_stream());
        let response = HttpResponse::Ok()
            .content_type(mime_type)
            // TODO: This likely will result in lots of byte copying in
            // production mode. Should implement our own MessageBody
            // for Cow<'static, [u8]>
            .body(bytes.into_owned());
        Ok(   
            response
        )
    }
} 


#[derive(RustEmbed, Debug)]
#[folder = "static/"]
struct StaticFiles;

#[derive(RustEmbed, Debug)]
#[folder = "web-client/src"]
struct WebClientFiles;

#[derive(RustEmbed, Debug)]
#[folder = "web-client/web_modules"]
struct WebClientDeps;


/// Routes that require a server with options.allow_login:
fn logged_in_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(
            resource("/post")
                .route(get().to(view_post))
                .route(post().to(post_post)),
        )
    ;
}


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
        .route("/web-cli/modules/{path:.*}", get().to(WebClientDeps::response))
        .route("/web-cli/{path:.*}", get().to(WebClientFiles::response))
    ;
}

fn index(backend: Data<Box<dyn Backend>>) -> Result<impl Responder, Error> {
    let response = IndexPage {
        name: "World".into(),
        hashes: backend.get_hashes()?,
    }
    .responder();

    Ok(response)
}

fn view_blob(
    backend: Data<Box<dyn Backend>>,
    path: Path<(String,)>,
) -> Result<impl Responder, Error> {
    let (base58hash,) = path.into_inner();
    let hash = Hash::from_base58(base58hash.as_ref())?;
    let result = backend.get_blob(&hash)?;
    let response = HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(result.unwrap_or("No result.".into()));
    Ok(response)
}

fn view_md(
    backend: Data<Box<dyn Backend>>,
    path: Path<(String,)>,
) -> Result<impl Responder, Error> {
    let (base58hash,) = path.into_inner();
    let hash = Hash::from_base58(base58hash.as_ref())?;
    let result = backend.get_blob(&hash)?.unwrap_or("No result.".into());
    let result = String::from_utf8(result)?;

    let parser = pulldown_cmark::Parser::new(&result);
    use pulldown_cmark::Event::*;
    let parser = parser.map(|event| match event {
        Html(value) => Code(value),
        InlineHtml(value) => Text(value),
        x => x,
    });

    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);

    let response = HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html);
    Ok(response)
}

fn view_post() -> impl Responder {
    PostPage::default().responder()
}

fn post_post(
    form: Form<PostForm>,
    backend: Data<Box<dyn Backend>>,
) -> Result<impl Responder, Error> {
    let form = form.into_inner();
    let hash = backend.save_blob(form.body.as_bytes())?;

    let url = format!("/blob/{}", hash.to_base58());

    let response = HttpResponse::SeeOther().header("location", url).finish();
    Ok(response)
}

fn file_not_found() -> impl Responder {
    NotFoundPage {}
        .responder()
        .with_status(StatusCode::NOT_FOUND)
}



#[derive(Template, Default)]
#[template(path = "login.html")]
struct LoginPage {
    logged_in_pkey: Option<String>,
}

#[derive(Deserialize, Default)]
struct LoginForm {
    secret_key: String,
}


#[derive(Template, Default)]
#[template(path = "logged_out.html")]
struct LoggedOutPage {}

fn create_id() -> impl Responder {
    let pair = crate::crypto::SigKeyPair::new();
    CreateIDPage {
        public_key: pair.public().bytes().to_base58(),
        secret_key: pair.secret().bytes().to_base58(),
    }
    .responder()
}

#[derive(Template, Default)]
#[template(path = "create_id.html")]
struct CreateIDPage {
    public_key: String,
    secret_key: String,
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
    form: PostForm,
}

#[derive(Deserialize, Default)]
struct PostForm {
    body: String,
}
