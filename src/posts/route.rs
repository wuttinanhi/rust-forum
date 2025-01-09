use actix_session::Session;
use actix_web::{
    get, post,
    web::{self},
    HttpResponse, Responder,
};

use handlebars::Handlebars;
use serde_json::json;

use crate::{
    comments::{
        repository::{delete_comment, list_comments_with_user},
        types::CommentPublic,
    },
    db::{DbError, DbPool},
    posts::{dto::CreatePostFormData, repository::delete_post, types::PostPublic},
    utils::{
        flash::{handle_flash_message, set_flash_message},
        handlebars_helper::update_handlebars_data,
        http::create_redirect,
        pagination::{build_handlebars_pagination_result, QueryPagination},
        session::handlebars_add_user,
        users::get_session_user,
    },
};

use crate::posts::repository::get_post_with_user;

use super::repository::{create_post, get_post, list_post_with_user};

#[get("/create")]
pub async fn create_post_route(
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let mut data = json!({
        "parent": "base"
    });

    handlebars_add_user(&session, &mut data);
    handle_flash_message(&mut data, &session);

    let body = hb.render("posts/create", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[post("/create")]
pub async fn create_post_submit_route(
    pool: web::Data<DbPool>,
    form: web::Form<CreatePostFormData>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let user = get_session_user(&session)?;

    let create_post_result = web::block(move || {
        let mut conn = pool.get()?;
        create_post(&mut conn, &user.id, &form.title, &form.body)
    })
    .await?;

    match create_post_result {
        Ok(new_post) => {
            let new_post_url = format!("/posts/{}", new_post.id);

            set_flash_message(&session, "success", "Created post!")?;

            Ok(create_redirect(&new_post_url))
        }

        Err(_) => {
            set_flash_message(&session, "error", "Failed to create post!")?;

            Ok(create_redirect(""))
        }
    }
}

#[get("/{post_id}")]
pub async fn view_post_route(
    pool: web::Data<DbPool>,
    hb: web::Data<Handlebars<'_>>,
    path: web::Path<i32>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let post_id = path.into_inner();
    let mut hb_data = json!({ "parent": "base" });

    // Combine database operations into single block
    let data_result: Result<(PostPublic, Vec<CommentPublic>), DbError> = web::block(move || {
        let mut conn = pool.get()?;

        let post = get_post_with_user(&mut conn, post_id)
            .map_err(|e| DbError::from(format!("Failed to get post: {}", e)))?;

        let comments = list_comments_with_user(&mut conn, &post.post.id)
            .map_err(|e| DbError::from(format!("Failed to get comments: {}", e)))?;

        Ok((post, comments))
    })
    .await?;

    match data_result {
        Ok((post, comments)) => {
            dbg!(&post);

            update_handlebars_data(&mut hb_data, "post", json!(post));
            update_handlebars_data(&mut hb_data, "comments", json!(comments));
        }

        Err(e) => {
            let error_msg = format!("Failed to get post: {}", e);
            set_flash_message(&session, "error", &error_msg)?;
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/"))
                .finish());
        }
    }

    handlebars_add_user(&session, &mut hb_data);
    handle_flash_message(&mut hb_data, &session);

    let body = hb
        .render("posts/view", &hb_data)
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(body))
}

// #[get("/")]
pub async fn index_list_posts_route(
    pagination_data: QueryPagination,
    pool: web::Data<DbPool>,
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let mut data = json!({
        "parent": "base",
    });

    let pagination_data_clone = pagination_data.clone();
    let posts_result = web::block(move || {
        let mut conn = pool.get()?;
        list_post_with_user(&mut conn, &pagination_data_clone)
    })
    .await?;

    match posts_result {
        Ok(result) => {
            update_handlebars_data(&mut data, "posts", json!(&result.posts));

            let pagination_result = build_handlebars_pagination_result(
                result.total,
                pagination_data.page,
                pagination_data.limit,
            );

            update_handlebars_data(&mut data, "pagination_result", json!(pagination_result));
        }

        Err(_) => set_flash_message(&session, "error", "failed to list posts")?,
    }

    handlebars_add_user(&session, &mut data);
    handle_flash_message(&mut data, &session);

    let body = hb.render("posts/index", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[post("/delete/{post_id}")]
pub async fn delete_post_route(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let post_id = path.into_inner();
    let session_user = get_session_user(&session)?;

    let delete_post_result = web::block(move || {
        let mut conn = pool.get()?;

        let post = get_post(&mut conn, post_id)
            .map_err(|e| DbError::from(format!("failed to get post {}", e)))?;

        if post.user_id != session_user.id {
            return Err(DbError::from("User does not own post"));
        }

        delete_post(&mut conn, post_id)
    })
    .await?;

    match delete_post_result {
        Ok(_) => set_flash_message(&session, "success", "Deleted post")?,

        Err(why) => {
            set_flash_message(&session, "error", &format!("Failed to delete post {}", why))?
        }
    }

    Ok(create_redirect("/"))
}
