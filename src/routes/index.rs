use actix_session::Session;
use actix_web::{get, web, Responder};
use handlebars::Handlebars;
use serde_json::json;

use crate::utils::flash::handle_flash_message;
use crate::utils::session::handle_session_user;

#[get("/")]
pub async fn index(hb: web::Data<Handlebars<'_>>, session: Session) -> impl Responder {
    let mut data = json!({
        "parent": "base",
    });

    handle_session_user(&session, &mut data);
    handle_flash_message(&mut data, &session);

    let body = hb.render("index", &data).unwrap();
    web::Html::new(body)
}
