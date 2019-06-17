use std::io;

use actix_web::{App, Responder, HttpServer};
use actix_web::web::{self, Path, resource, HttpResponse as Response};

use askama::Template;

use webbrowser;

mod responder_util;
use responder_util::ToResponder;


fn index() -> impl Responder {
    IndexPage{
        name: "World".into()
    }.responder()
}

fn main() -> io::Result<()> {
    let server = HttpServer::new(|| App::new().service(
        resource("/").to(index))
    ).bind("127.0.0.1:8080")?;

    let url = "http://127.0.0.1:8080/";
    let opened = webbrowser::open(url);
    if !opened.is_ok() {
        println!("Warning: Couldn't open browser.");
    }
    println!("Started at: {}", url);

    server.run()
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
    body: String
}
