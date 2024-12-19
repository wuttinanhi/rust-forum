use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct HelloQuery {
    pub name: String,
}

#[get("/query")]
pub async fn hello_query(query: web::Query<HelloQuery>) -> impl Responder {
    HttpResponse::Ok().body(format!("hello {}", query.name))
}
