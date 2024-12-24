use actix_session::Session;
use actix_web::{
    http::header,
    post,
    web::{self},
    HttpResponse, Responder,
};

use serde::Deserialize;

use crate::{
    establish_connection,
    utils::{flash::set_flash_message, users::get_session_user},
};

use super::crud::create_comment;

#[derive(Deserialize)]
struct CreateCommentFormData {
    post_id: i32,
    body: String,
}

#[post("/create")]
pub async fn create_comment_submit_route(
    form: web::Form<CreateCommentFormData>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let user = get_session_user(&session)?;
    let conn = &mut establish_connection();

    let comment_result = create_comment(conn, &user.id, &form.post_id, &form.body);

    let redirect_url = format!("/posts/{}#{}", comment_result.post_id, comment_result.id);

    set_flash_message(&session, "success", "Created comment!")?;

    Ok(HttpResponse::Found()
        .insert_header((header::LOCATION, redirect_url))
        .finish())
}
