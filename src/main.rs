use std::io;

use actix_web::{App, Responder, HttpServer};
use actix_web::web::{self, Path, Form, resource, HttpResponse as Response};
use actix_web_codegen::{get, post};

use serde::Deserialize;

use askama::Template;

use webbrowser;

mod responder_util;
use responder_util::ToResponder;



fn main() -> io::Result<()> {
    let server = HttpServer::new(|| App::new()
        .service(index)
        .service(view_post)
        .service(post)
    ).bind("127.0.0.1:8080")?;

    let url = "http://127.0.0.1:8080/";
    let opened = webbrowser::open(url);
    if !opened.is_ok() {
        println!("Warning: Couldn't open browser.");
    }
    println!("Started at: {}", url);

    server.run()
}

#[get("/")]
fn index() -> impl Responder {
    IndexPage{
        name: "World".into()
    }.responder()
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
