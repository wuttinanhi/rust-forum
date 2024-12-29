use actix_session::Session;
use actix_web::{
    get, post,
    web::{self, Redirect},
    HttpResponse, Responder,
};

use handlebars::Handlebars;
use serde_json::json;

use crate::{
    db::DbPool,
    users::{
        constants::SESSION_KEY_USER,
        crud::{create_user, login_user},
        dto::{UserLoginFormData, UserRegisterFormData},
        types::{user_to_user_session, SessionUser},
    },
    utils::{flash::set_flash_message, http::create_redirect},
};

#[get("/login")]
pub async fn users_login_route(
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    if session.get::<SessionUser>(SESSION_KEY_USER)?.is_some() {
        set_flash_message(&session, "error", "User already logged in!")?;

        return Ok(create_redirect("/"));
    }

    let data = json!({
        "parent": "base"
    });

    let body = hb.render("users/login", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[post("/login")]
pub async fn users_login_post_route(
    pool: web::Data<DbPool>,
    form: web::Form<UserLoginFormData>,
    session: Session,
    hb: web::Data<Handlebars<'_>>,
) -> actix_web::Result<impl Responder> {
    // use web::block to offload blocking Diesel queries without blocking server thread
    let user_result = web::block(move || {
        // note that obtaining a connection from the pool is also potentially blocking
        let mut conn = pool.get()?;

        login_user(&mut conn, &form.email, &form.password)
    })
    .await?;

    // map diesel query errors to a 500 error response

    match user_result {
        Ok(user) => {
            let user_session = user_to_user_session(&user);

            session.insert(SESSION_KEY_USER, user_session)?;

            Ok(create_redirect("/"))
        }

        Err(_) => {
            let data = json!({
                "parent": "base",
                "error": "Error invalid credentials"
            });

            let body = hb.render("users/login", &data).unwrap();

            Ok(HttpResponse::Ok().body(body))
        }
    }
}

#[get("/register")]
pub async fn users_register_route(
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    if session.get::<SessionUser>(SESSION_KEY_USER)?.is_some() {
        set_flash_message(&session, "error", "User already logged in!")?;

        return Ok(create_redirect("/"));
    }

    let data = json!({
        "parent": "base"
    });

    let body = hb.render("users/register", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[post("/register")]
pub async fn users_register_post_route(
    pool: web::Data<DbPool>,
    form: web::Form<UserRegisterFormData>,
    session: Session,
    hb: web::Data<Handlebars<'_>>,
) -> actix_web::Result<impl Responder> {
    let hb_data = json!({
        "parent": "base",
    });

    let create_user_result = web::block(move || {
        let mut conn = pool.get()?;
        create_user(&mut conn, &form.name, &form.email, &form.password)
    })
    .await?;

    if create_user_result.is_err() {
        let body = hb.render("users/register", &hb_data).unwrap();

        return Ok(HttpResponse::Ok().body(body));
    }

    set_flash_message(&session, "success", "Created user. you can now login!")?;

    Ok(create_redirect("/"))
}

#[get("/logout")]
pub async fn users_logout(session: Session) -> actix_web::Result<impl Responder> {
    // clear session
    session.clear();

    set_flash_message(&session, "success", "Logout Successfully!")?;

    Ok(Redirect::to("/"))
}
