use actix_web::{
    get,
    http::header,
    post,
    web::{self},
    HttpResponse, Responder,
};

use handlebars::Handlebars;
use serde::Deserialize;
use serde_json::json;

use crate::{
    db::map_diesel_error_to_message,
    establish_connection,
    users::crud::{create_user, login_user},
};

#[get("/login")]
pub async fn users_login(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    let data = json!({
        "parent": "users/base"
    });
    let body = hb.render("users/login", &data).unwrap();

    web::Html::new(body)
}

#[derive(Deserialize)]
struct UserLoginFormData {
    email: String,
    password: String,
}

#[post("/login")]
pub async fn users_login_post(
    form: web::Form<UserLoginFormData>,
    hb: web::Data<Handlebars<'_>>,
) -> actix_web::Result<impl Responder> {
    let conn = &mut establish_connection();
    let user = login_user(conn, &form.email, &form.password);

    match user {
        Ok(_) => Ok(HttpResponse::Found()
            .insert_header((header::LOCATION, "/"))
            .finish()),

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
pub async fn users_register(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    let data = json!({
        "parent": "users/base"
    });
    let body = hb.render("users/register", &data).unwrap();

    web::Html::new(body)
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
