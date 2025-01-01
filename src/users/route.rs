use std::{
    fs::File,
    io::{Read, Write},
};

use actix_multipart::form::MultipartForm;
use actix_session::Session;
use actix_web::{
    error, get, post,
    web::{self},
    HttpResponse, Responder,
};

use handlebars::Handlebars;
use serde_json::json;

use crate::{
    db::{DbError, DbPool},
    users::{
        constants::SESSION_KEY_USER,
        crud::{
            create_user, get_user_by_id, get_user_sanitized, login_user, update_user_data,
            update_user_password, validate_user_password,
        },
        dto::{
            UserChangePasswordFormData, UserLoginFormData, UserRegisterFormData,
            UserUpdateFormData, UserUploadProfilePictureForm,
        },
        types::{user_to_user_session, SessionUser},
    },
    utils::{
        flash::{handle_flash_message, set_flash_message, FLASH_ERROR, FLASH_SUCCESS},
        handlebars_helper::update_handlebars_data,
        http::create_redirect,
        users::get_session_user,
    },
    validate_input_user_name, validate_input_user_password,
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
    validate_input_user_password!(&form.password);

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

    validate_input_user_name!(&form.name);
    validate_input_user_password!(&form.password);

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

    Ok(create_redirect("/"))
}

#[get("/settings")]
pub async fn users_settings_route(
    pool: web::Data<DbPool>,
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let session_user = get_session_user(&session)?;

    let mut hb_data = json!({
        "parent": "base"
    });

    let user = web::block(move || {
        let mut conn = pool.get()?;

        // we need to get updated data from db

        get_user_sanitized(&mut conn, session_user.id)
            .map_err(|_| DbError::from("User by session not found"))
    })
    .await?;

    match user {
        Ok(user) => update_handlebars_data(&mut hb_data, "user", json!(user)),
        Err(why) => set_flash_message(&session, FLASH_ERROR, &why.to_string())?,
    }

    handle_flash_message(&mut hb_data, &session);

    let body = hb
        .render("users/settings", &hb_data)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

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

    validate_input_user_password!(&form.confirm_password);

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

        update_user_password(&mut conn, &user, new_password)
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

#[post("/update")]
pub async fn users_update_data_post_route(
    pool: web::Data<DbPool>,
    form: web::Form<UserUpdateFormData>,
    session: Session,
    // hb: web::Data<Handlebars<'_>>,
) -> actix_web::Result<impl Responder> {
    let session_user = get_session_user(&session)?;

    validate_input_user_name!(&form.new_name);

    let user = web::block(move || {
        let mut conn = pool.get()?;

        // get db user
        let user = get_user_by_id(&mut conn, session_user.id)
            .map_err(|_| DbError::from("User by session not found"))?;

        // update user data
        update_user_data(&mut conn, &user, &form.new_name)
    })
    .await?;

    match user {
        Ok(_) => set_flash_message(&session, FLASH_SUCCESS, "Updated user data!")?,

        Err(why) => set_flash_message(
            &session,
            FLASH_ERROR,
            &format!("Failed to update user data! : {why}"),
        )?,
    }

    Ok(create_redirect("/users/settings"))
}

#[post("/profilepicture")]
pub async fn users_profile_picture_post_route(
    pool: web::Data<DbPool>,
    session: Session,
    MultipartForm(form): MultipartForm<UserUploadProfilePictureForm>,
) -> actix_web::Result<impl Responder> {
    let content_type = form.profile_picture.content_type;

    if let Some(mime) = content_type {
        let content_type_string = mime.to_string();

        if content_type_string != "image/jpeg" && content_type_string != "image/png" {
            return Err(actix_web::error::ErrorBadRequest("Invalid file type"));
        }

        let size = form.profile_picture.size;
        let size10mb = 10485760;

        if size > size10mb {
            return Err(actix_web::error::ErrorBadRequest("File too large"));
        }

        let file: Result<File, std::io::Error> = web::block(|| {
            let file_bytes: Vec<u8> = form
                .profile_picture
                .file
                .bytes()
                .collect::<Result<_, _>>()?;

            let mut file = std::fs::File::create("test.jpg")?;

            file.write_all(&file_bytes)?;
            Ok(file)
        })
        .await?;

        dbg!("file uploaded");
        dbg!(file?);

        set_flash_message(&session, FLASH_SUCCESS, "File uploaded")?;
    } else {
        return Err(actix_web::error::ErrorBadRequest("No file uploaded"));
    }

    Ok(create_redirect("/users/settings"))
}
