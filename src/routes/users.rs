use actix_session::Session;
use actix_web::{
    get,
    http::header,
    post,
    web::{self, Redirect},
    HttpResponse, Responder,
};

use handlebars::Handlebars;
use serde::Deserialize;
use serde_json::json;

use crate::{
    db::map_diesel_error_to_message,
    establish_connection,
    users::crud::{create_user, login_user},
    utils::flash::set_flash_message,
};

#[get("/login")]
pub async fn users_login(
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    if session.get::<i32>("user_id")?.is_some() {
        set_flash_message(&session, "error", "User already logged in!")?;

        return Ok(HttpResponse::Found()
            .insert_header((header::LOCATION, "/"))
            .finish());
    }

    let data = json!({
        "parent": "users/base"
    });

    let body = hb.render("users/login", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[derive(Deserialize)]
struct UserLoginFormData {
    email: String,
    password: String,
}

#[post("/login")]
pub async fn users_login_post(
    form: web::Form<UserLoginFormData>,
    session: Session,
    hb: web::Data<Handlebars<'_>>,
) -> actix_web::Result<impl Responder> {
    let conn = &mut establish_connection();
    let user_result = login_user(conn, &form.email, &form.password);

    match user_result {
        Ok(user) => {
            session.insert("user_id", user.id)?;

            Ok(HttpResponse::Found()
                .insert_header((header::LOCATION, "/"))
                .finish())
        }

        Err(_) => {
            let data = json!({
                "parent": "users/base",
                "error": "Error login"
            });

            let body = hb.render("users/login", &data).unwrap();

            Ok(HttpResponse::Ok().body(body))
        }
    }
}

#[get("/register")]
pub async fn users_register(
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    if session.get::<i32>("user_id")?.is_some() {
        set_flash_message(&session, "error", "User already logged in!")?;

        return Ok(HttpResponse::Found()
            .insert_header((header::LOCATION, "/"))
            .finish());
    }

    let data = json!({
        "parent": "users/base"
    });

    let body = hb.render("users/register", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[derive(Deserialize)]
struct UserRegisterFormData {
    name: String,
    email: String,
    password: String,
}

#[post("/register")]
pub async fn users_register_post(
    form: web::Form<UserRegisterFormData>,
    hb: web::Data<Handlebars<'_>>,
) -> actix_web::Result<impl Responder> {
    let conn = &mut establish_connection();

    let result = create_user(conn, &form.name, &form.email, &form.password)
        .map(|_| {
            json!({
               "parent": "users/base",
               "success": "Created user"
            })
        })
        .unwrap_or_else(|e| {
            json!({
                "parent": "users/base",
                "error": format!("Error create user: {}", map_diesel_error_to_message(e))
            })
        });

    let body = hb.render("users/register", &result).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[get("/logout")]
pub async fn users_logout(session: Session) -> actix_web::Result<impl Responder> {
    // clear session
    session.clear();

    set_flash_message(&session, "success", "Logout Successfully!")?;

    Ok(Redirect::to("/"))
}
