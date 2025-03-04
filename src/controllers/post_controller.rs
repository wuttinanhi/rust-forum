use actix_session::Session;
use actix_web::{
    get, post,
    web::{self},
    HttpRequest, HttpResponse, Responder,
};

use handlebars::Handlebars;
use serde_json::json;

use crate::{
    db::WebError,
    entities::{
        comment::ListCommentResult,
        post::{PostFormData, PostPublic},
    },
    utils::{
        flash::{handle_flash_message, set_flash_message, FLASH_ERROR, FLASH_SUCCESS},
        handlebars_helper::update_handlebars_data,
        http::{create_redirect, redirect_back},
        pagination::{build_handlebars_pagination_result, QueryPagination},
        session::handlebars_add_user,
        users::get_session_user,
    },
    AppKit,
};

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
    app_kit: web::Data<AppKit>,
    form: actix_web_validator::Form<PostFormData>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let user = get_session_user(&session)?;

    let create_post_result = web::block(move || {
        app_kit
            .post_repository
            .create_post(user.id, &form.title, &form.body)
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
    app_kit: web::Data<AppKit>,
    hb: web::Data<Handlebars<'_>>,
    path: web::Path<i32>,
    session: Session,
    pagination: QueryPagination,
) -> actix_web::Result<impl Responder> {
    let post_id = path.into_inner();
    let mut hb_data = json!({ "parent": "base" });
    let session_user = get_session_user(&session);

    let pagination_clone = pagination.clone();

    let data_result: Result<(PostPublic, ListCommentResult), WebError> = web::block(move || {
        let post = app_kit
            .post_service
            .get_post_with_user(post_id)
            .map_err(|e| WebError::from(format!("Failed to get post: {}", e.to_string())))?;

        let comments = app_kit
            .comment_service
            .get_comments_with_user(post.post.id, &pagination_clone)
            .map_err(|e| WebError::from(format!("Failed to get comments: {}", e.to_string())))?;

        Ok((post, comments))
    })
    .await?;

    match data_result {
        Ok((mut post, mut comment_result)) => {
            // if post.user_id is equal session user id then allow update
            if let Ok(user) = session_user {
                if post.user.id == user.id {
                    post.allow_update = true;
                }

                comment_result.comments.iter_mut().for_each(|c| {
                    if c.user.id == user.id {
                        c.allow_update = true;
                    }
                });
            }

            let comment_pagination_result =
                build_handlebars_pagination_result(comment_result.total, &pagination);

            update_handlebars_data(
                &mut hb_data,
                "pagination_result",
                json!(comment_pagination_result),
            );

            update_handlebars_data(&mut hb_data, "title", json!(post.post.title));
            update_handlebars_data(&mut hb_data, "post", json!(post));
            update_handlebars_data(&mut hb_data, "comments_result", json!(comment_result));
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
    app_kit: web::Data<AppKit>,
    pagination: QueryPagination,
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let mut data = json!({
        "parent": "base",
    });

    let pagination_data_clone = pagination.clone();
    let posts_result = web::block(move || {
        app_kit
            .post_service
            .get_posts_with_user(&pagination_data_clone)
    })
    .await?;

    match posts_result {
        Ok(result) => {
            update_handlebars_data(&mut data, "posts_result", json!(&result));

            let pagination_result = build_handlebars_pagination_result(result.total, &pagination);

            update_handlebars_data(&mut data, "pagination_result", json!(pagination_result));
        }

        Err(why) => set_flash_message(
            &session,
            "error",
            &format!("failed to list posts: {}", &why.to_string(),),
        )?,
    }

    handle_flash_message(&mut data, &session);
    update_handlebars_data(&mut data, "title", json!("Posts"));
    let _ = handlebars_add_user(&session, &mut data);

    let body = hb.render("posts/index", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[get("/update/{post_id}")]
pub async fn update_post_route(
    app_kit: web::Data<AppKit>,
    req: HttpRequest,
    hb: web::Data<Handlebars<'_>>,
    session: Session,
    path: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let session_user = get_session_user(&session)?;

    let post_id = path.into_inner();

    let post = web::block(move || app_kit.post_service.get_post_with_user(post_id))
        .await?
        .map_err(|_| {
            actix_web::error::ErrorInternalServerError("failed to get target update post")
        })?;

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
pub async fn update_post_submit_route(
    app_kit: web::Data<AppKit>,
    req: HttpRequest,
    form: actix_web_validator::Form<PostFormData>,
    path: web::Path<i32>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let post_id = path.into_inner();
    let session_user = get_session_user(&session)?;

    let update_post_result = web::block(move || {
        let fetch_result = app_kit
            .post_service
            .get_post_with_user(post_id)
            .map_err(|e| WebError::from(format!("failed to get post {}", e)))?;

        if fetch_result.user.id != session_user.id {
            return Err(WebError::from("User does not own post"));
        }

        let post = app_kit
            .post_service
            .update_post(fetch_result.post.id, &form.title, &form.body)
            .map_err(|e| WebError::from(format!("failed to update post {}", e)))?;

        Ok(post)
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
    app_kit: web::Data<AppKit>,
    req: HttpRequest,
    path: web::Path<i32>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let post_id = path.into_inner();
    let session_user = get_session_user(&session)?;

    let delete_post_result = web::block(move || {
        let post = app_kit
            .post_service
            .get_post(post_id)
            .map_err(|e| WebError::from(format!("failed to get post {}", e)))?;

        if post.user_id != session_user.id {
            return Err(WebError::from("User does not own post"));
        }

        let row_affected = app_kit
            .post_service
            .delete_post(post_id)
            .map_err(|e| WebError::from(format!("failed to delete post {}", e)))?;

        Ok(row_affected)
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
