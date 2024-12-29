use actix_session::Session;
use actix_web::{
    get, post,
    web::{self},
    HttpResponse, Responder,
};

use handlebars::Handlebars;
use serde_json::json;

use crate::{
    comments::crud::list_comments_with_user,
    db::{DbError, DbPool},
    posts::dto::{CreatePostFormData, PostPageData},
    users::crud::get_user_sanitized,
    utils::{
        flash::{handle_flash_message, set_flash_message},
        handlebars_helper::update_handlebars_data,
        http::create_redirect,
        session::handle_session_user,
        users::get_session_user,
    },
};

use super::crud::{create_post, get_post, list_post_with_user};

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

    let mut data = json!({
        "parent": "base",
    });

    let post_data = web::block(move || {
        let mut conn = pool.get()?;
        let post = get_post(&mut conn, post_id)?;
        let author = get_user_sanitized(&mut conn, post.user_id).ok_or(diesel::NotFound)?;
        let comments = list_comments_with_user(&mut conn, &post.id)?;

        Ok::<PostPageData, DbError>(PostPageData {
            post,
            author,
            comments,
        })
    })
    .await?;

    match post_data {
        Ok(post_data) => {
            update_handlebars_data(
                &mut data,
                "post",
                serde_json::to_value(&post_data.post).unwrap(),
            );
            update_handlebars_data(
                &mut data,
                "post_user",
                serde_json::to_value(post_data.author).unwrap(),
            );

            update_handlebars_data(
                &mut data,
                "comments",
                serde_json::to_value(&post_data.comments).unwrap(),
            );
        }
        Err(_) => {
            set_flash_message(&session, "error", "failed to get post")?;
        }
    }

    handle_session_user(&session, &mut data);
    handle_flash_message(&mut data, &session);

    let body = hb.render("posts/view", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[get("/")]
pub async fn index_list_posts_route(
    pool: web::Data<DbPool>,
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let list_posts_result = web::block(move || {
        let mut conn = pool.get()?;
        list_post_with_user(&mut conn)
    })
    .await?;

    let mut data = json!({
        "parent": "base",
    });

    match list_posts_result {
        Ok(posts) => {
            update_handlebars_data(&mut data, "posts", serde_json::to_value(&posts).unwrap());
        }
        Err(_) => set_flash_message(&session, "error", "failed to list posts")?,
    }

    handle_session_user(&session, &mut data);
    handle_flash_message(&mut data, &session);

    let body = hb.render("posts/index", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}
