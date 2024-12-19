use actix_web::{get, web, Responder};
use handlebars::Handlebars;
use serde_json::json;

#[get("/")]
pub async fn index(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    let data = json!({
        "parent": "base"
    });
    let body = hb.render("index", &data).unwrap();
    web::Html::new(body)
}
