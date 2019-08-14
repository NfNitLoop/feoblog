use actix_web::{App, Responder, HttpServer};
use actix_web::web::{self, Path, Form, Data, resource, route, HttpResponse, get, post};
use askama::Template;
use serde::Deserialize;
use failure::{Error, bail, ResultExt};

use in_memory_session::{Session, SessionReader, SessionWriter};
use crate::responder_util::ToResponder;
use crate::backend::{self, *};

pub fn cmd_open() -> Result<(), Error> {
    rust_sodium::init();

    let factory = backend::sqlite::Factory::new("feoblog.sqlite3".into());
    factory.open()?.setup().context("Error setting up DB")?;
    let middleware = in_memory_session::Middleware::new();

    let app_factory = move || {
        let db = factory.open().expect("Couldn't open DB connection.");
        App::new()
            .wrap(middleware.clone())
            .data(db)
            .route("/", get().to(index))
            .service(
                resource("/post")
                .route(get().to(view_post))
                .route(post().to(post_post))
            )
            .route("/blob/{base58hash}", get().to(view_blob))
            .route("/md/{base58hash}", get().to(view_md))
            .route("/sessionTest", get().to(session_test))
            .default_service(route().to(file_not_found))
    };
    
    let server = HttpServer::new(app_factory)
        .bind("127.0.0.1:8080")?
    ;

    let url = "http://127.0.0.1:8080/";
    let opened = webbrowser::open(url);
    if !opened.is_ok() {
        println!("Warning: Couldn't open browser.");
    }
    println!("Started at: {}", url);

    server.run()?; // Actually blocks & runs forever.

    Ok(())
}



fn index(
        backend: Data<Box<dyn Backend>>
) -> Result<impl Responder, Error> {
    let response = IndexPage{
        name: "World".into(),
        hashes: backend.get_hashes()?,
    }.responder();

    Ok(response)
}

fn view_blob(
    backend: Data<Box<dyn Backend>>,
    path: Path<(String,)>
) -> Result<impl Responder, Error> 
{
    let (base58hash,) = path.into_inner();
    let hash = Hash::from_base58(base58hash.as_ref())?;
    let result = backend.get_blob(&hash)?;
    let response = HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(result.unwrap_or("No result.".into()))
    ;
    Ok(response)
}

fn view_md(
    backend: Data<Box<dyn Backend>>,
    path: Path<(String,)>
) -> Result<impl Responder, Error> 
{
    let (base58hash,) = path.into_inner();
    let hash = Hash::from_base58(base58hash.as_ref())?;
    let result = backend.get_blob(&hash)?
        .unwrap_or("No result.".into())
    ;
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
        .body(html)
    ;
    Ok(response)
}

fn view_post() -> impl Responder {
    PostPage::default().responder()
}

fn post_post(
    form: Form<PostForm>,
    backend: Data<Box<dyn Backend>>
) -> Result<impl Responder, Error>
{
    let form = form.into_inner();
    let hash = backend.save_blob(form.body.as_bytes())?;

    let url = format!("/blob/{}", hash.to_base58());

    let response = HttpResponse::SeeOther()
        .header("location", url)
        .finish()
    ;
    
    Ok(response)
}

fn session_test(session: Session) -> Result<impl Responder, Error>
{
    let mut writer = session.write();
    let mut count = writer.get("counter").unwrap_or(0 as u32);
    count = count + 1;
    writer.set("counter", count);

    return Ok(count.to_string());
}

fn file_not_found() -> impl Responder {
    NotFoundPage{}.responder()
}

#[derive(Template)] 
#[template(path = "not_found.html")]
struct NotFoundPage
{
}

#[derive(Template)] 
#[template(path = "index.html")]
struct IndexPage
{
    name: String,
    hashes: Vec<Hash>
}

#[derive(Template, Default)]
#[template(path = "post.html")]
struct PostPage
{
    form: PostForm
}

#[derive(Deserialize, Default)]
struct PostForm {
    body: String
}
