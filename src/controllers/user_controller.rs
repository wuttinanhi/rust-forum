use actix_multipart::form::MultipartForm;
use actix_session::Session;
use actix_web::{
    error, get, post,
    web::{self},
    HttpRequest, HttpResponse, Responder,
};

use handlebars::Handlebars;
use serde_json::json;

use crate::{
    db::WebError,
    entities::user::{user_to_user_public, validate_user_password, SESSION_KEY_USER},
    models::UpdateUserNameAndProfilePicture,
    utils::{
        flash::{handle_flash_message, set_flash_message, FLASH_ERROR, FLASH_SUCCESS},
        handlebars_helper::update_handlebars_data,
        http::{create_redirect, redirect_back},
        users::get_session_user,
    },
    validate_password_and_confirm_password, AppKit,
};

use crate::entities::user::{
    UserChangePasswordFormData, UserLoginFormData, UserPasswordResetRequest,
    UserPasswordResetTokenQueryString, UserPasswordResetTokenRequest, UserRegisterFormData,
    UserUpdateFormData, UserUploadProfilePictureForm,
};

#[get("/login")]
pub async fn users_login_route(
    hb: web::Data<Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    if get_session_user(&session).is_ok() {
        set_flash_message(&session, "error", "User already logged in!")?;
        return Ok(create_redirect("/"));
    }

    let mut hb_data = json!({
        "parent": "base"
    });

    update_handlebars_data(&mut hb_data, "title", json!("Login"));
    handle_flash_message(&mut hb_data, &session);
    let body = hb.render("users/login", &hb_data).unwrap();

    // dbg!(&hb_data);

    Ok(HttpResponse::Ok().body(body))
}

#[post("/login")]
pub async fn users_login_post_route(
    app_kit: web::Data<AppKit>,
    form: actix_web_validator::Form<UserLoginFormData>,
    session: Session,
    hb: web::Data<Handlebars<'_>>,
    req: HttpRequest,
) -> actix_web::Result<impl Responder> {
    let mut data = json!({
        "parent": "base",
    });

    // get what client send
    // dbg!(&form);

    // let cf_turnstile_response = form
    //     .cf_turnstile_response
    //     .to_owned()
    //     .unwrap_or("".to_string());
    // let turnstile_result =
    //     crate::utils::turnstile::validate_turnstile_wrapper(&cf_turnstile_response).await;
    // if let Err(turnstile_error) = turnstile_result {
    //     crate::utils::flash::set_flash_message(
    //         &session,
    //         crate::utils::flash::FLASH_ERROR,
    //         &turnstile_error.message,
    //     )?;
    //     return Ok(crate::utils::http::redirect_back(&req));
    // }

    crate::validate_turnstile_field!(form, session, req);

    let login_result =
        web::block(move || app_kit.user_service.login_user(&form.email, &form.password))
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    match login_result {
        Ok(user) => {
            let user_public = user_to_user_public(&user);

            // set session user value
            session.insert(SESSION_KEY_USER, user_public)?;

            return Ok(create_redirect("/"));
        }

        Err(_) => {
            update_handlebars_data(&mut data, "error", json!("Invalid login"));
        }
    }

    let body = hb.render("users/login", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
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
    req: HttpRequest,
) -> actix_web::Result<impl Responder> {
    let mut hb_data = json!({
        "parent": "base",
    });

    crate::validate_turnstile_field!(form, session, req);

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
    let session_user = get_session_user(&session).map_err(|e| {
        dbg!("get session user err", &e);
        e
    })?;

    dbg!(&session_user);

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

        Err(why) => {
            dbg!(&why);

            // set_flash_message(&session, FLASH_ERROR, &why.to_string())?
            session.clear();

            return Ok(create_redirect("/"));
        }
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
    form: actix_web_validator::Form<UserChangePasswordFormData>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let session_user = get_session_user(&session)?;

    validate_password_and_confirm_password!(form);

    let user = web::block(move || {
        // get db user
        let user = app_kit
            .user_service
            .get_user_by_id(session_user.id)
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
        .new_profile_picture
        .content_type
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid MIME type"))?;

    let content_type_string = content_type.to_string();
    if content_type_string != "image/jpeg" && content_type_string != "image/png" {
        return Err(actix_web::error::ErrorBadRequest("Invalid file type"));
    }

    // check file size
    let size = form.new_profile_picture.size;
    if size > 10 * 1024 * 1024 {
        return Err(actix_web::error::ErrorBadRequest("File too large"));
    }

    // generate save file path
    let static_file_dir_path =
        std::env::var("STATIC_FILE_DIR").unwrap_or("/app/static".to_string());
    let unix_time = chrono::Utc::now().timestamp();
    let filename = format!("{}.jpg", unix_time);
    let static_file_path = format!("{static_file_dir_path}/{filename}");

    let session_user = get_session_user(&session)?;

    web::block(move || {
        dbg!(&static_file_path);

        // save file to static file directory

        // fixing Invalid cross-device link (os error 18)
        // this happens when we try to rename a file to different filesystem
        // we fix by persist the file to a temporary location first
        // then copy it to the static file directory and delete the temporary file
        let tmp_file_path = format!("/tmp/{unix_time}.jpg");
        form.new_profile_picture
            .file
            .persist(&tmp_file_path)
            .map_err(WebError::from)?;

        // copy the file to the static file directory
        std::fs::copy(&tmp_file_path, &static_file_path).map_err(|e| {
            WebError::from(format!("Failed to copy file to static directory: {}", e))
        })?;

        // delete the temporary file
        std::fs::remove_file(&tmp_file_path)
            .map_err(|e| WebError::from(format!("Failed to delete temporary file: {}", e)))?;

        println!(
            "saved profile picture of user {} to {}",
            session_user.id, static_file_path
        );

        // add slash to make it accessible from client
        let public_static_url = format!("/static/{}", &filename);

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
                    user_profile_picture_url: Some(&public_static_url),
                },
            )
            .map_err(|e| WebError::from(e.to_string()))?;

        println!("user profile picture uploaded: {}", &public_static_url);

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
    crate::validate_turnstile_field!(form, session, req);

    let _ = web::block(move || {
        // let mut conn = pool.get()?;

        let target_reset_password_user = app_kit
            .user_service
            .get_user_by_email(&form.email)
            .map_err(|_| WebError::from("failed to get user"))?;

        let password_reset = app_kit
            .token_service
            .create_password_reset(target_reset_password_user.id)
            .map_err(|_| WebError::from("failed to create password reset record"))?;

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

        app_kit
            .email_service
            .send_email(
                &target_reset_password_user.email,
                "Password reset instruction - Rust Forum",
                &email_body,
            )
            .map_err(|_| WebError::from("failed to send reset password email"))?;

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
        // get password reset record
        let password_reset = app_kit
            .token_service
            .get_password_reset_by_token(&form.token)
            .map_err(|_| WebError::from("failed to get password reset record"))?;

        // consume password reset record and update user password
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
