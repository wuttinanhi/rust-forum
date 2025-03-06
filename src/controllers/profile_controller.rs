use std::sync::{Arc, Mutex};

use actix_session::Session;
use actix_web::{
    http::header::ContentType, web, FromRequest, HttpRequest, HttpResponse, Responder,
};
use futures::future::{ready, Ready};
use handlebars::Handlebars;
use serde_json::json;

use crate::{
    db::WebError,
    entities::{comment::CommentPublic, post::PostPublic},
    utils::{
        flash::handle_flash_message,
        handlebars_helper::update_handlebars_data,
        pagination::{
            build_handlebars_pagination_result, HandlebarsPaginationResult, QueryPagination,
        },
        session::handlebars_add_user,
    },
    AppKit,
};

pub struct OptionalFetchMode(pub String);

impl FromRequest for OptionalFetchMode {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let fetch_mode = req
            .match_info()
            .get("fetch_mode")
            .map(|s| s.to_string())
            .unwrap_or("posts".to_string());

        // unwrap_or("posts".to_string());

        ready(Ok(OptionalFetchMode(fetch_mode)))
    }
}

// #[get("/profile/{user_id}/{fetch_mode:.*}")]
pub async fn profile_view_route(
    _: HttpRequest,
    app_kit: web::Data<AppKit>,
    session: Session,
    path: web::Path<(i32,)>,
    fetch_mode: OptionalFetchMode,
    pagination: QueryPagination,
    hb: web::Data<Handlebars<'_>>,
) -> actix_web::Result<impl Responder> {
    let user_id = path.into_inner().0;
    let fetch_mode = fetch_mode.0;
    let fetch_mode_clone = fetch_mode.clone();

    let mut hb_data = json!({
        "parent": "base",
    });

    let user_created_posts: Arc<Mutex<Vec<PostPublic>>> = Arc::new(Mutex::new(vec![]));
    let user_created_comments: Arc<Mutex<Vec<CommentPublic>>> = Arc::new(Mutex::new(vec![]));
    let pagination_result: Arc<Mutex<HandlebarsPaginationResult>> =
        Arc::new(Mutex::new(HandlebarsPaginationResult::default()));

    let post_service_cloned = app_kit.post_service.clone();
    let comment_service_cloned = app_kit.comment_service.clone();

    let user_created_posts_cloned = user_created_posts.clone();
    let user_created_comments_cloned = user_created_comments.clone();
    let pagination_result_cloned = pagination_result.clone();

    let user_data = web::block(move || {
        let user_sanitized = app_kit
            .user_service
            .get_user_by_id_public(user_id)
            .map_err(|_| WebError::from("Failed to get comments"))?;

        // get_user_sanitized_by_id(&mut conn, user_id)?;

        if fetch_mode_clone == "posts" {
            let created_posts = post_service_cloned
                .get_posts_by_user(user_sanitized.id, &pagination)
                .map_err(|_| WebError::from("Failed to get posts by user"))?;
            // (&mut conn, user_id, &pagination)?;

            user_created_posts_cloned
                .lock()
                .unwrap()
                .extend(created_posts.posts);
            // .unwrap()
            // .extend(created_posts.posts);

            *(pagination_result_cloned.lock().unwrap()) =
                build_handlebars_pagination_result(created_posts.total, &pagination);
        } else if fetch_mode_clone == "comments" {
            let created_comments = comment_service_cloned
                .get_comments_by_user(user_sanitized.id, &pagination)
                .map_err(|_| WebError::from("Failed to get comments"))?;

            user_created_comments_cloned
                .lock()
                .unwrap()
                .extend(created_comments.comments);

            *pagination_result_cloned.lock().unwrap() =
                build_handlebars_pagination_result(created_comments.total, &pagination);
        } else {
            return Err(WebError::from("no fetch mode was provide"));
        }

        Ok(user_sanitized)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;

    update_handlebars_data(&mut hb_data, "profile_users", json!(user_data));
    update_handlebars_data(
        &mut hb_data,
        "title",
        json!(format!("Profile {}", user_data.name)),
    );

    if fetch_mode == "posts" || fetch_mode.is_empty() {
        let profile_users_created_posts = &*user_created_posts.lock().unwrap();

        update_handlebars_data(
            &mut hb_data,
            "profile_users_created_posts",
            json!(profile_users_created_posts),
        );

        update_handlebars_data(&mut hb_data, "fetch_mode_posts", json!(true));
    } else if fetch_mode == "comments" {
        let profile_users_created_comments = &*user_created_comments.lock().unwrap();
        update_handlebars_data(
            &mut hb_data,
            "profile_users_created_comments",
            json!(profile_users_created_comments),
        );

        update_handlebars_data(&mut hb_data, "fetch_mode_comments", json!(true));
    }

    let pagination_result_deref = &*(pagination_result.lock().unwrap());

    update_handlebars_data(
        &mut hb_data,
        "pagination_result",
        json!(pagination_result_deref),
    );

    handlebars_add_user(&session, &mut hb_data)?;
    handle_flash_message(&mut hb_data, &session);

    let body = hb
        .render("users/profile", &hb_data)
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
