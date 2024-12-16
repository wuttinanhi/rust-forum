use std::fmt::format;

use actix_web::{
    get, post,
    web::{self, Query},
    App, HttpResponse, HttpServer, Responder,
};
use serde::Deserialize;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[derive(Deserialize)]
struct HelloQuery {
    name: String,
}

#[get("/query")]
async fn hello_query(query: Query<HelloQuery>) -> impl Responder {
    HttpResponse::Ok().body(format!("hello {}", query.name))
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = "127.0.0.1";
    let port = 8080;

    println!("listening at http://{}:{}", host, port);

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(hello_query)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind((host, port))?
    .run()
    .await
}
