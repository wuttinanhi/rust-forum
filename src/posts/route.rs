use actix_session::Session;
use actix_web::{
    get, post,
    web::{self},
    HttpRequest, HttpResponse, Responder,
};

use handlebars::Handlebars;
use serde_json::json;

use crate::{
    comments::{repository::get_comments_with_user, types::CommentPublic},
    db::{DbError, DbPool},
    posts::{
        dto::{CreatePostFormData, UpdatePostFormData},
        repository::{delete_post, update_post},
        types::PostPublic,
    },
    utils::{
        flash::{handle_flash_message, set_flash_message, FLASH_ERROR, FLASH_SUCCESS},
        handlebars_helper::update_handlebars_data,
        http::{create_redirect, redirect_back},
        pagination::{build_handlebars_pagination_result, QueryPagination},
        session::handlebars_add_user,
        users::get_session_user,
    },
};

use crate::posts::repository::get_post_with_user;

use super::repository::{create_post, get_post, get_posts_with_user};

#[get("/create")]
pub async fn create_post_route(
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let mut data = json!({
        "parent": "base",
        "title": "Create new post",
        "form_action": "/posts/create",
        "form_header": "Create new post",
        "form_submit_button_text": "Create",
    });

    handle_flash_message(&mut data, &session);
    handlebars_add_user(&session, &mut data)?;

    let body = hb.render("posts/form", &data).unwrap();

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
    let session_user = get_session_user(&session);

    // Combine database operations into single block
    let data_result: Result<(PostPublic, Vec<CommentPublic>), DbError> = web::block(move || {
        let mut conn = pool.get()?;

        let post = get_post_with_user(&mut conn, post_id)
            .map_err(|e| DbError::from(format!("Failed to get post: {}", e)))?;

        let comments = get_comments_with_user(&mut conn, post.post.id)
            .map_err(|e| DbError::from(format!("Failed to get comments: {}", e)))?;

        Ok((post, comments))
    })
    .await?;

    match data_result {
        Ok((mut post, mut comments)) => {
            // if post.user_id is equal session user id then allow update
            if let Ok(user) = session_user {
                if post.user.id == user.id {
                    post.allow_update = true;
                }

                comments.iter_mut().for_each(|c| {
                    if c.user.id == user.id {
                        c.allow_update = true;
                    }
                });
            }

            update_handlebars_data(&mut hb_data, "title", json!(post.post.title));
            update_handlebars_data(&mut hb_data, "post", json!(post));
            update_handlebars_data(&mut hb_data, "comments", json!(comments));
        }

        Err(e) => {
            update_handlebars_data(&mut hb_data, "title", json!("error"));
            let error_msg = format!("Error : {}", e);
            set_flash_message(&session, "error", &error_msg)?;

            return Ok(HttpResponse::Found()
                .append_header(("Location", "/"))
                .finish());
        }
    }

    let _ = handlebars_add_user(&session, &mut hb_data);
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
        get_posts_with_user(&mut conn, &pagination_data_clone)
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

    handle_flash_message(&mut data, &session);
    update_handlebars_data(&mut data, "title", json!("Posts"));
    let _ = handlebars_add_user(&session, &mut data);

    let body = hb.render("posts/index", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[get("/update/{post_id}")]
pub async fn update_post_route(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    hb: web::Data<Handlebars<'_>>,
    session: Session,
    path: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let session_user = get_session_user(&session)?;

    let post_id = path.into_inner();

    let post = web::block(move || {
        let mut conn = pool.get()?;
        get_post_with_user(&mut conn, post_id)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;

    // check if user able to update post
    if post.user.id != session_user.id {
        set_flash_message(&session, FLASH_ERROR, "User does not own post")?;
        return Ok(redirect_back(&req));
    }

    let mut data = json!({
        "parent": "base",
        "title": format!("Update post : {}", post.post.title),
        "form_header": format!("Update post : {}", post.post.title),
          "form_action": format!("/posts/update/{}", post.post.id),
        "form_submit_button_text": "Update",
    });

    update_handlebars_data(&mut data, "post", json!(post.post));
    handle_flash_message(&mut data, &session);
    handlebars_add_user(&session, &mut data)?;

    let body = hb.render("posts/form", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[post("/update/{post_id}")]
pub async fn update_post_post_route(
    req: HttpRequest,
    form: web::Form<UpdatePostFormData>,
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let post_id = path.into_inner();
    let session_user = get_session_user(&session)?;

    let update_post_result = web::block(move || {
        let mut conn = pool.get()?;

        let fetch_result = get_post_with_user(&mut conn, post_id)
            .map_err(|e| DbError::from(format!("failed to get post {}", e)))?;

        if fetch_result.user.id != session_user.id {
            return Err(DbError::from("User does not own post"));
        }

        update_post(&mut conn, fetch_result.post.id, &form.title, &form.body)
    })
    .await?;

    match update_post_result {
        Ok(post) => {
            set_flash_message(&session, FLASH_SUCCESS, "Post updated")?;
            Ok(create_redirect(&format!("/posts/{}", post.id)))
        }

        Err(why) => {
            set_flash_message(
                &session,
                FLASH_ERROR,
                &format!("Failed to update post {}", why),
            )?;

            Ok(redirect_back(&req))
        }
    }
}

#[post("/delete/{post_id}")]
pub async fn delete_post_route(
    req: HttpRequest,
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
        Ok(_) => {
            set_flash_message(&session, FLASH_SUCCESS, "Post deleted")?;
            Ok(create_redirect("/"))
        }

        Err(why) => {
            set_flash_message(
                &session,
                FLASH_ERROR,
                &format!("Failed to delete post {}", why),
            )?;

            Ok(redirect_back(&req))
        }
    }
}
