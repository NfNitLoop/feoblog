// Disable warnings in dev mode.
#![cfg_attr(debug_assertions, allow(dead_code, unused))]

use std::io;

use actix_web::{App, Responder, HttpServer};
use actix_web::web::{self, Path, Form, Data, resource, HttpResponse as Response};
use actix_web_codegen::{get, post};

use serde::Deserialize;

use askama::Template;

use webbrowser;

mod responder_util;
use responder_util::ToResponder;

mod backend;
use backend::*;


fn main() -> Result<(), failure::Error> {

    let factory = backend::sqlite::Factory::new("feoblog.sqlite3".into());
    factory.open()?.setup()?;

    let app_factory = move || {
        let db = factory.open().expect("Couldn't open DB connection.");
        App::new()
            .data(db)
            .service(index)
            .service(view_post)
            .service(post)
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


struct FeoBlog
{
    backend: Box<dyn Backend>
}

#[get("/")]
fn index(backend: Data<Box<dyn Backend>>) -> Result<impl Responder, failure::Error> {

    let result = backend.get("homepage".as_bytes())?;
    
    let response = Response::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(result.unwrap_or("No result.".into()))
    ;
    Ok(response)

    // Ok(
    //     IndexPage{
    //         name: "World".into()
    //     }.responder()
    // )
}

#[get("/post")]
fn view_post() -> impl Responder {
    PostPage::default().responder()
}

#[post("/post")]
fn post(form: Form<PostForm>) -> impl Responder {
    IndexPage{
        name: form.into_inner().body
    }.responder()
}


#[derive(Template)] 
#[template(path = "index.html")]
struct IndexPage
{
    name: String
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
