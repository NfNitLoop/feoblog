// Disable warnings in dev mode.
#![cfg_attr(debug_assertions, allow(dead_code, unused))]

#[cfg(test)]
mod tests;

use std::io;

use actix_web::{App, Responder, HttpServer};
use actix_web::web::{self, Path, Form, Data, resource, HttpResponse};
use actix_web_codegen::{get, post};
use serde::Deserialize;
use askama::Template;
use failure::Error;
use structopt::StructOpt;

use in_memory_session::{Session, SessionReader, SessionWriter};

mod responder_util;
use responder_util::ToResponder;

mod backend;
use backend::*;

fn main() -> Result<(), Error> {
    let command = Command::from_args();
    use Command::*;

    match command {
        Open{..} => return cmd_open(),
        Crypto{..} => return test_crypto(),
        other => { 
            println!("Unimplemented: {:?}", other);
            return Ok(());
        }
    };
}

fn cmd_open() -> Result<(), Error> {
    rust_sodium::init();

    let factory = backend::sqlite::Factory::new("feoblog.sqlite3".into());
    factory.open()?.setup()?;
    let middleware = in_memory_session::Middleware::new();

    let app_factory = move || {
        let db = factory.open().expect("Couldn't open DB connection.");
        App::new()
            .wrap(middleware.clone())
            .data(db)
            .service(index)
            .service(view_post)
            .service(view_blob)
            .service(view_md)
            .service(post)
            .service(session_test)
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

#[derive(StructOpt, Debug)]
#[structopt(
    name="feoblog",
    about="A distributed P2P (micro|macro)blog system.",
    author="",
)]
enum Command
{
    #[structopt(name="open")]
    /// Open a browser window to locally view/add content.
    Open { },

    #[structopt(name="serve")]
    /// Serve local content as a web site.
    /// The write UI is disabled. Content must be signed and pushed from
    /// other instances.
    Serve { },

    #[structopt(
        name="crypto",
        raw(setting = "structopt::clap::AppSettings::Hidden"),
    )]
    /// Test some crypto primitives.
    Crypto { },
}

fn test_crypto() -> Result<(), failure::Error>
{
    use rust_sodium::crypto::box_;
    use rust_base58::*;

    let (ourpk, oursk) = box_::gen_keypair();
    let vecpk: Vec<u8> = ourpk[..].into();
    let vecsk: Vec<u8> = oursk[..].into();
    println!("pk: {}, bytes: {}", ourpk[..].to_base58(), vecpk.len());
    println!("sk: {}, bytes: {}", oursk[..].to_base58(), vecsk.len());

    use rust_sodium::crypto::scalarmult::curve25519::Scalar;
    use rust_sodium::crypto::scalarmult::curve25519::scalarmult_base;

    let s = Scalar::from_slice(&oursk[..]).expect("scalar");
    let group_element = scalarmult_base(&s);
    println!("derived pk: {}", group_element[..].to_base58());


    use rust_sodium::crypto::sign::*;
    println!("PUBLICKEYBYTES: {}", PUBLICKEYBYTES);
    println!("SECRETKEYBYTES: {}", SECRETKEYBYTES);
    println!("SIGNATUREBYTES: {}", SIGNATUREBYTES);

    return Ok(());
}


struct FeoBlog
{
    backend: Box<dyn Backend>
}

#[get("/")]
fn index(
        backend: Data<Box<dyn Backend>>
) -> Result<impl Responder, failure::Error> {
    let response = IndexPage{
        name: "World".into(),
        hashes: backend.get_hashes()?,
    }.responder();

    Ok(response)
}

#[get("/blob/{base58hash}")]
fn view_blob(
    backend: Data<Box<dyn Backend>>,
    path: Path<(String,)>
) -> Result<impl Responder, failure::Error> 
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

#[get("/md/{base58hash}")]
fn view_md(
    backend: Data<Box<dyn Backend>>,
    path: Path<(String,)>
) -> Result<impl Responder, failure::Error> 
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

#[get("/post")]
fn view_post() -> impl Responder {
    PostPage::default().responder()
}

#[post("/post")]
fn post(
    form: Form<PostForm>,
    backend: Data<Box<dyn Backend>>
) -> Result<impl Responder, failure::Error>
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

#[get("/sessionTest")]
fn session_test(session: Session) -> Result<impl Responder, failure::Error>
{
    let mut writer = session.write();
    let mut count = writer.get("counter").unwrap_or(0 as u32);
    count = count + 1;
    writer.set("counter", count);

    return Ok(count.to_string());
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
