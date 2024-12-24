use actix_session::Session;
use actix_web::{
    get,
    http::header,
    post,
    web::{self},
    HttpResponse, Responder,
};

use handlebars::Handlebars;
use serde::Deserialize;
use serde_json::json;

use crate::{
    establish_connection,
    users::crud::get_user_sanitized,
    utils::{
        flash::{handle_flash_message, set_flash_message},
        handlebars_helper::update_handlebars_data,
        session::handle_session_user,
    },
};

use super::crud::{create_post, get_post};

#[get("/create")]
pub async fn create_post_route(
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let mut data = json!({
        "parent": "base"
    });

    handle_session_user(&session, &mut data);
    handle_flash_message(&mut data, &session);

    let body = hb.render("posts/create", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[derive(Deserialize)]
struct CreatePostFormData {
    title: String,
    body: String,
}

#[post("/create")]
pub async fn create_post_submit_route(
    form: web::Form<CreatePostFormData>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let conn = &mut establish_connection();
    let create_post_resultx = create_post(conn, &1, &form.title, &form.body);

    let new_post_url = format!("/posts/{}", create_post_resultx.id);

    set_flash_message(&session, "success", "Created post!")?;

    Ok(HttpResponse::Found()
        .insert_header((header::LOCATION, new_post_url))
        .finish())
}

#[get("/{post_id}")]
pub async fn view_post_route(
    hb: web::Data<Handlebars<'_>>,
    path: web::Path<i32>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let post_id = path.into_inner();

    let conn = &mut establish_connection();
    let post = get_post(conn, post_id);

    let mut data = json!({
        "parent": "base",
    });

    match post {
        Some(post) => {
            let post_user = get_user_sanitized(conn, post.user_id);

            update_handlebars_data(&mut data, "post", serde_json::to_value(post).unwrap());
            update_handlebars_data(
                &mut data,
                "post_user",
                serde_json::to_value(post_user).unwrap(),
            );
        }
        None => set_flash_message(&session, "error", "Post not found!")?,
    }

    handle_session_user(&session, &mut data);
    handle_flash_message(&mut data, &session);

    let body = hb.render("posts/view", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}
