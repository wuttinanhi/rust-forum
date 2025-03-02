
use actix_multipart::form::MultipartForm;
use actix_session::Session;
use actix_web::{
    error, get, post,
    web::{self}, HttpRequest, HttpResponse, Responder,
};

use handlebars::Handlebars;
use serde_json::json;

use crate::{
    db::{DbPool, WebError},
    models::UpdateUserNameAndProfilePicture,
    users::{constants::SESSION_KEY_USER, types::user_to_user_public},
    utils::{
        flash::{handle_flash_message, set_flash_message, FLASH_ERROR, FLASH_SUCCESS},
        handlebars_helper::update_handlebars_data,
        http::{create_redirect, redirect_back},
        users::get_session_user,
    },
    validate_password_and_confirm_password, AppKit,
};

use super::dto::{
    UserChangePasswordFormData, UserLoginFormData, UserPasswordResetRequest,
    UserPasswordResetTokenQueryString, UserPasswordResetTokenRequest, UserRegisterFormData,
    UserUpdateFormData, UserUploadProfilePictureForm,
};

use super::repository::{get_user_by_id, validate_user_password};

#[get("/login")]
pub async fn users_login_route(
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    if get_session_user(&session).is_ok() {
        set_flash_message(&session, "error", "User already logged in!")?;

        return Ok(create_redirect("/"));
    }

    let mut data = json!({
        "parent": "base"
    });

    update_handlebars_data(&mut data, "title", json!("Login"));
    handle_flash_message(&mut data, &session);
    let body = hb.render("users/login", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[post("/login")]
pub async fn users_login_post_route(
    app_kit: web::Data<AppKit>,
    form: actix_web_validator::Form<UserLoginFormData>,
    session: Session,
    hb: web::Data<Handlebars<'_>>,
) -> actix_web::Result<impl Responder> {
    let login_result =
        web::block(move || app_kit.user_service.login_user(&form.email, &form.password))
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    match login_result {
        Ok(user) => {
            let user_public = user_to_user_public(&user);

            // set session user value
            session.insert(SESSION_KEY_USER, user_public)?;

            Ok(create_redirect("/"))
        }

        Err(_) => {
            let data = json!({
                "parent": "base",
                "error": "Invalid login"
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
    if get_session_user(&session).is_ok() {
        set_flash_message(&session, "error", "User already logged in!")?;

        return Ok(create_redirect("/"));
    }

    let mut data = json!({
        "parent": "base"
    });

    update_handlebars_data(&mut data, "title", json!("Register"));
    handle_flash_message(&mut data, &session);

    let body = hb.render("users/register", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[post("/register")]
pub async fn users_register_post_route(
    app_kit: web::Data<AppKit>,
    form: actix_web_validator::Form<UserRegisterFormData>,
    session: Session,
    hb: web::Data<Handlebars<'_>>,
) -> actix_web::Result<impl Responder> {
    let mut hb_data = json!({
        "parent": "base",
    });

    let create_user_result = web::block(move || {
        app_kit
            .user_service
            .register_user(&form.name, &form.email, &form.password)
    })
    .await?;

    match create_user_result {
        Ok(_) => {
            set_flash_message(&session, "success", "Created user. you can now login!")?;

            Ok(create_redirect("/"))
        }

        Err(_) => {
            set_flash_message(&session, FLASH_ERROR, "Failed to register user.")?;

            handle_flash_message(&mut hb_data, &session);

            let body = hb.render("users/register", &hb_data).unwrap();
            Ok(HttpResponse::Ok().body(body))
        }
    }
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
    app_kit: web::Data<AppKit>,
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let session_user = get_session_user(&session)?;

    let mut hb_data = json!({
        "title": "User settings",
        "parent": "base"
    });

    let user_public_result = web::block(move || {
        // // we need to get updated data from db
        // get_user_sanitized_by_id(&mut conn, session_user.id)
        //     .map_err(|_| WebError::from("User by session not found"))

        app_kit.user_service.get_user_by_id_public(session_user.id)
    })
    .await?;

    match user_public_result {
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
    app_kit: web::Data<AppKit>,
    pool: web::Data<DbPool>,
    form: actix_web_validator::Form<UserChangePasswordFormData>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let session_user = get_session_user(&session)?;

    validate_password_and_confirm_password!(form);

    let user = web::block(move || {
        let mut conn = pool.get()?;

        // get db user
        let user = get_user_by_id(&mut conn, session_user.id)
            .map_err(|_| WebError::from("User by session not found"))?;

        // validate current password
        let current_password_valid = validate_user_password(&user, &form.current_password);
        if !current_password_valid {
            return Err(WebError::from("Invalid current password!"));
        }

        // update user password
        let new_password = &form.confirm_password;

        // update_user_password(&mut conn, &user, new_password)
        //     .map_err(|_| WebError::from("Failed to change password!"))

        app_kit
            .user_service
            .update_user_password(user.id, new_password)
            .map_err(|_| WebError::from("failed to update user password"))
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
    app_kit: web::Data<AppKit>,
    form: actix_web_validator::Form<UserUpdateFormData>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let session_user = get_session_user(&session)?;

    let user = web::block(move || {
        // get db user
        let user = app_kit.user_service.get_user_by_id(session_user.id)?;

        // update user data
        app_kit.user_service.update_user_data(
            user.id,
            &UpdateUserNameAndProfilePicture {
                name: Some(&form.new_name),
                user_profile_picture_url: None,
            },
        )
    })
    .await?;

    match user {
        Ok(_) => set_flash_message(&session, FLASH_SUCCESS, "Updated user data")?,

        Err(why) => set_flash_message(
            &session,
            FLASH_ERROR,
            &format!("Failed to update user data! : {why}"),
        )?,
    }

    Ok(create_redirect("/users/settings"))
}

#[post("/profilepicture")]
pub async fn users_profile_picture_upload_post_route(
    app_kit: web::Data<AppKit>,
    session: Session,
    MultipartForm(form): MultipartForm<UserUploadProfilePictureForm>,
) -> actix_web::Result<impl Responder> {
    // check content type
    let content_type = form
        .profile_picture
        .content_type
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid MIME type"))?;

    let content_type_string = content_type.to_string();
    if content_type_string != "image/jpeg" && content_type_string != "image/png" {
        return Err(actix_web::error::ErrorBadRequest("Invalid file type"));
    }

    // check file size
    let size = form.profile_picture.size;
    if size > 10 * 1024 * 1024 {
        return Err(actix_web::error::ErrorBadRequest("File too large"));
    }

    // generate save file path
    let unix_time = chrono::Utc::now().timestamp();
    let file_path = format!("static/{}.jpg", unix_time);

    let session_user = get_session_user(&session)?;

    web::block(move || {
        // save file
        form.profile_picture
            .file
            .persist(&file_path)
            .map_err(WebError::from)?;

        // add slash to make it accessible from client
        let user_profile_picture_url_pre_slash = format!("/{}", &file_path);

        let db_user = app_kit
            .user_service
            .get_user_by_id(session_user.id)
            .map_err(|_| WebError::from("User by session not found"))?;

        // update user data with new profile image url
        app_kit
            .user_service
            .update_user_data(
                db_user.id,
                &UpdateUserNameAndProfilePicture {
                    name: None,
                    user_profile_picture_url: Some(&user_profile_picture_url_pre_slash),
                },
            )
            .map_err(|e| WebError::from(e.to_string()))?;

        println!(
            "user profile picture uploaded: {}",
            &user_profile_picture_url_pre_slash
        );

        Ok::<_, WebError>(())
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;

    set_flash_message(&session, FLASH_SUCCESS, "Profile picture uploaded")?;

    Ok(create_redirect("/users/settings"))
}

#[get("/resetpassword")]
pub async fn users_resetpassword_route(
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    if get_session_user(&session).is_ok() {
        set_flash_message(&session, "error", "User already logged in!")?;

        return Ok(create_redirect("/"));
    }

    let mut data = json!({
        "title": "Reset password",
        "parent": "base"
    });

    handle_flash_message(&mut data, &session);

    let body = hb.render("users/resetpassword", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[post("/resetpassword")]
pub async fn users_resetpassword_post_route(
    app_kit: web::Data<AppKit>,
    form: actix_web_validator::Form<UserPasswordResetRequest>,
    session: Session,
    req: HttpRequest,
) -> actix_web::Result<impl Responder> {
    let _ = web::block(move || {
        // let mut conn = pool.get()?;

        let target_reset_password_user = app_kit
            .user_service
            .get_user_by_email(&form.email)
            .map_err(|_| WebError::from("failed to get user"))?;

        let password_reset = app_kit
            .token_service
            .create_password_reset(target_reset_password_user.id)?;

        #[allow(non_snake_case)]
        let APP_DOMAIN_URL = std::env::var("APP_DOMAIN_URL").expect("APP_DOMAIN_URL must be set");

        let password_reset_url = format!(
            "{}/users/resetpasswordtoken?token={}",
            APP_DOMAIN_URL, password_reset.reset_token
        );

        let email_body = format!(
            "you requested to reset password:\n<a href=\"{}\">{}</a>",
            password_reset_url, password_reset_url
        );

        app_kit.email_service.send_email(
            &target_reset_password_user.email,
            "Password reset instruction - Rust Forum",
            &email_body,
        )?;

        Ok::<_, WebError>(())
    })
    .await?;

    set_flash_message(
        &session,
        FLASH_SUCCESS,
        "An email with password reset instruction was sent!",
    )?;

    Ok(redirect_back(&req))
}

#[get("/resetpasswordtoken")]
pub async fn users_resetpasswordtoken_route(
    hb: web::Data<Handlebars<'_>>,
    session: Session,
    query: web::Query<UserPasswordResetTokenQueryString>,
) -> actix_web::Result<impl Responder> {
    let mut data = json!({
        "title": "Reset password with Token",
        "parent": "base",
        "token": query.token,
    });

    handle_flash_message(&mut data, &session);

    let body = hb.render("users/resetpasswordtoken", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[post("/resetpasswordtoken")]
pub async fn users_resetpasswordtoken_post_route(
    app_kit: web::Data<AppKit>,
    form: actix_web_validator::Form<UserPasswordResetTokenRequest>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    validate_password_and_confirm_password!(form);

    let result: Result<(), WebError> = web::block(move || {
        let password_reset = app_kit
            .token_service
            .get_password_reset_by_token(&form.token)?;

        app_kit
            .user_service
            .update_user_password_from_reset(&password_reset, &form.new_password)
            .map_err(|e| WebError::from(e.to_string()))?;

        Ok(())
    })
    .await?;

    match result {
        Ok(_) => set_flash_message(&session, FLASH_SUCCESS, "Successfully reset password!")?,

        Err(why) => set_flash_message(
            &session,
            FLASH_ERROR,
            &format!("failed to reset password: {}", why),
        )?,
    };

    Ok(create_redirect("/users/login"))
}
