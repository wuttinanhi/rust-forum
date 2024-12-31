use actix_session::Session;
use actix_web::{
    get, post,
    web::{self, Redirect},
    HttpResponse, Responder,
};

use handlebars::Handlebars;
use serde_json::json;

use crate::{
    db::{DbError, DbPool},
    users::{
        constants::SESSION_KEY_USER,
        crud::{
            change_user_password, create_user, get_user_by_id, login_user, validate_user_password,
        },
        dto::{UserChangePasswordFormData, UserLoginFormData, UserRegisterFormData},
        types::{user_to_user_session, SessionUser},
    },
    utils::{
        flash::{handle_flash_message, set_flash_message, FLASH_ERROR, FLASH_SUCCESS},
        http::create_redirect,
        users::get_session_user,
    },
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

#[post("/logout")]
pub async fn users_logout(session: Session) -> actix_web::Result<impl Responder> {
    // clear session
    session.clear();

    set_flash_message(&session, "success", "Logout Successfully!")?;

    Ok(Redirect::to("/"))
}

#[get("/settings")]
pub async fn users_settings_route(
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let user = get_session_user(&session)?;

    let mut hb_data = json!({
        "parent": "base",
        "user": user,
    });

    handle_flash_message(&mut hb_data, &session);

    let body = hb.render("users/settings", &hb_data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[post("/changepassword")]
pub async fn users_changepassword_post_route(
    pool: web::Data<DbPool>,
    form: web::Form<UserChangePasswordFormData>,
    session: Session,
    // hb: web::Data<Handlebars<'_>>,
) -> actix_web::Result<impl Responder> {
    let session_user = get_session_user(&session)?;

    let user = web::block(move || {
        let mut conn = pool.get()?;

        let new_password_and_confirm_password_equal = form.new_password == form.confirm_password;
        if !new_password_and_confirm_password_equal {
            return Err(DbError::from(
                "New password and confirm password not equal!",
            ));
        }

        // get db user
        let user = get_user_by_id(&mut conn, session_user.id)
            .map_err(|_| DbError::from("User by session not found"))?;

        // validate current password
        let current_password_valid = validate_user_password(&user, &form.current_password);
        if !current_password_valid {
            return Err(DbError::from("Invalid current password!"));
        }

        // update user password
        let new_password = &form.confirm_password;

        change_user_password(&mut conn, &user, new_password)
            .map_err(|_| DbError::from("Failed to change password!"))
    })
    .await?;

    match user {
        Ok(_) => set_flash_message(&session, FLASH_SUCCESS, "Change user password completed!")?,
        Err(why) => set_flash_message(
            &session,
            FLASH_ERROR,
            &format!("Failed to change user password! : {why}"),
        )?,
    }

    Ok(create_redirect("/users/settings"))
}
