use actix_session::Session;
use actix_web::{
    get, post,
    web::{self},
    HttpRequest, HttpResponse, Responder,
};

use handlebars::Handlebars;
use serde_json::json;

use crate::{
    comments::repository::get_page_where_comment_at,
    db::{DbPool, WebError},
    models::Comment,
    utils::{
        flash::{handle_flash_message, set_flash_message, FLASH_ERROR, FLASH_SUCCESS},
        handlebars_helper::update_handlebars_data,
        http::{create_redirect, redirect_back},
        session::handlebars_add_user,
        users::get_session_user,
    },
};

use super::dto::{CreateCommentFormData, UpdateCommentFormData};
use super::repository::{create_comment, delete_comment, get_comment, update_comment};

#[post("/create")]
pub async fn create_comment_submit_route(
    pool: web::Data<DbPool>,
    form: actix_web_validator::Form<CreateCommentFormData>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let user = get_session_user(&session).map_err(actix_web::error::ErrorInternalServerError)?;

    let post_id = form.post_id;

    let result: Result<(Comment, i64), WebError> = web::block(move || {
        let mut conn = pool.get()?;
        let comment = create_comment(&mut conn, &user.id, &form.post_id, &form.body)?;

        let target_comment_page = get_page_where_comment_at(&mut conn, &comment, 10)?;

        Ok((comment, target_comment_page))
    })
    .await?;

    match result {
        Ok((comment, target_comment_page)) => {
            let redirect_url = format!(
                "/posts/{}?page={}&per_page={}#{}",
                comment.post_id, target_comment_page, 10, comment.id
            );
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

#[get("/update/{comment_id}")]
pub async fn update_comment_route(
    req: HttpRequest,
    pool: actix_web::web::Data<DbPool>,
    hb: web::Data<Handlebars<'_>>,
    session: Session,
    path: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let session_user = get_session_user(&session)?;

    let comment_id = path.into_inner();

    let comment = web::block(move || {
        let mut conn = pool.get()?;
        get_comment(&mut conn, comment_id)
    })
    .await?;

    let comment = match comment {
        Ok(comment) => comment,
        Err(why) => {
            set_flash_message(&session, FLASH_ERROR, &why.to_string())?;
            return Ok(redirect_back(&req));
        }
    };

    // check if user able to update comment
    if comment.user_id != session_user.id {
        set_flash_message(&session, FLASH_ERROR, "Error : User does not own comment")?;
        return Ok(redirect_back(&req));
    }

    let mut data = json!({
        "parent": "base",
        "title": format!("Update comment : #{}", comment.id),
        "form_header": format!("Update comment : #{}", comment.id),
        "form_action": format!("/comments/update/{}", comment.id),
        "form_submit_button_text": "Update",
    });

    update_handlebars_data(&mut data, "comment", json!(comment));
    handle_flash_message(&mut data, &session);
    handlebars_add_user(&session, &mut data)?;

    let body = hb.render("comments/form", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[post("/update/{post_id}")]
pub async fn update_comment_post_route(
    req: HttpRequest,
    form: actix_web_validator::Form<UpdateCommentFormData>,
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let comment_id = path.into_inner();
    let session_user = get_session_user(&session)?;

    let result: Result<(Comment, i64), WebError> = web::block(move || {
        let mut conn = pool.get()?;

        let comment = get_comment(&mut conn, comment_id)
            .map_err(|e| WebError::from(format!("failed to get comment {}", e)))?;

        if comment.user_id != session_user.id {
            return Err(WebError::from("Error : User does not own comment"));
        }

        let comment = update_comment(&mut conn, comment.id, &form.body)?;

        let target_comment_page = get_page_where_comment_at(&mut conn, &comment, 10)?;

        Ok((comment, target_comment_page))
    })
    .await?;

    match result {
        Ok((comment, target_comment_page)) => {
            set_flash_message(&session, FLASH_SUCCESS, "comment updated")?;

            let redirect_url = format!(
                "/posts/{}?page={}&per_page={}#{}",
                comment.post_id, target_comment_page, 10, comment.id
            );
            Ok(create_redirect(&redirect_url))
        }

        Err(why) => {
            set_flash_message(&session, FLASH_ERROR, &why.to_string())?;

            Ok(redirect_back(&req))
        }
    }
}

#[post("/delete/{comment_id}")]
pub async fn delete_comment_route(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let comment_id = path.into_inner();
    let session_user = get_session_user(&session)?;

    let delete_comment_result: Result<(Comment, i64), WebError> = web::block(move || {
        let mut conn = pool.get()?;

        let comment = get_comment(&mut conn, comment_id)
            .map_err(|e| WebError::from(format!("failed to get comment {}", e)))?;

        if comment.user_id != session_user.id {
            return Err(WebError::from("Error : User does not own comment"));
        }

        delete_comment(&mut conn, comment.id)?;

        let target_comment_page = get_page_where_comment_at(&mut conn, &comment, 10)?;

        Ok((comment, target_comment_page))
    })
    .await?;

    match delete_comment_result {
        Ok((comment, target_comment_page)) => {
            set_flash_message(&session, FLASH_SUCCESS, "comment deleted")?;

            let redirect_url = format!(
                "/posts/{}?page={}&per_page={}#{}",
                comment.post_id, target_comment_page, 10, comment.id
            );
            Ok(create_redirect(&redirect_url))
        }

        Err(why) => {
            set_flash_message(&session, FLASH_ERROR, &why.to_string())?;

            Ok(redirect_back(&req))
        }
    }
}
