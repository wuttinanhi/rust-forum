use actix_session::Session;
use actix_web::{
    post,
    web::{self},
    Responder,
};

use crate::{
    comments::dto::CreateCommentFormData,
    db::DbPool,
    utils::{flash::set_flash_message, http::create_redirect, users::get_session_user},
};

use super::crud::create_comment;

#[post("/create")]
pub async fn create_comment_submit_route(
    pool: web::Data<DbPool>,
    form: web::Form<CreateCommentFormData>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let user = get_session_user(&session)?;

    let post_id = form.post_id;

    let create_comment_result = web::block(move || {
        let mut conn = pool.get()?;
        create_comment(&mut conn, &user.id, &form.post_id, &form.body)
    })
    .await?;

    match create_comment_result {
        Ok(new_comment) => {
            let redirect_url = format!("/posts/{}#{}", new_comment.post_id, new_comment.id);
            set_flash_message(&session, "success", "Created comment!")?;
            Ok(create_redirect(&redirect_url))
        }
        Err(_) => {
            let redirect_url = format!("/posts/{}", post_id);

            set_flash_message(&session, "error", "Error creating comment!")?;

            Ok(create_redirect(&redirect_url))
        }
    }
}
