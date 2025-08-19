use std::sync::LazyLock;

use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use bcrypt::verify;
use serde::{Deserialize, Serialize};
use validator::Validate;

use regex::Regex;

use actix_web::{FromRequest, HttpRequest};
use chrono::NaiveDateTime;

use crate::models::User;

use futures::future::{ready, Ready};

static NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9 ]{2,16}$").unwrap());

pub const SESSION_KEY_USER: &str = "user";

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct UserLoginFormData {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(
        min = 8,
        max = 100,
        message = "Password must be at least 8 characters and max 100 long"
    ))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct UserRegisterFormData {
    #[validate(regex(
        path = *NAME_REGEX,
        message = "Name must contain only letters, numbers and spaces. length between 2 and 16."
    ))]
    pub name: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(
        min = 8,
        max = 100,
        message = "Password must be at least 8 characters and max 100 long"
    ))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct UserChangePasswordFormData {
    #[validate(length(
        min = 8,
        max = 100,
        message = "Current password must be at least 8 characters and max 100 long"
    ))]
    pub current_password: String,
    #[validate(length(
        min = 8,
        max = 100,
        message = "New password must be at least 8 characters and max 100 long"
    ))]
    pub new_password: String,
    #[validate(length(
        min = 8,
        max = 100,
        message = "Confirm password must be at least 8 characters and max 100 long"
    ))]
    pub confirm_password: String,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct UserUpdateFormData {
    #[validate(regex(
        path = *NAME_REGEX,
        message = "Name must contain only letters, numbers and spaces. length between 2 and 16."
    ))]
    pub new_name: String,
}

#[derive(Debug, MultipartForm)]
pub struct UserUploadProfilePictureForm {
    #[multipart(limit = "10MB")]
    pub profile_picture: TempFile,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct UserPasswordResetRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct UserPasswordResetTokenQueryString {
    #[validate(length(min = 10, max = 30, message = "Invalid token"))]
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct UserPasswordResetTokenRequest {
    #[validate(length(min = 10, max = 30, message = "Invalid token"))]
    pub token: String,
    #[validate(length(
        min = 8,
        max = 100,
        message = "New password must be at least 8 characters and max 100 long"
    ))]
    pub new_password: String,
    #[validate(length(
        min = 8,
        max = 100,
        message = "Confirm password must be at least 8 characters and max 100 long"
    ))]
    pub confirm_password: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct UserPublic {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub user_profile_picture_url: String,
}

pub fn user_to_user_public(user: &User) -> UserPublic {
    UserPublic {
        id: user.id,
        name: user.name.clone(),
        created_at: user.created_at,
        user_profile_picture_url: user.user_profile_picture_url.clone().unwrap_or(format!(
            "https://ui-avatars.com/api/?size=250&name={}",
            user.name
        )),
    }
}

pub struct OptionalFetchMode(pub String);

impl FromRequest for OptionalFetchMode {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let fetch_mode = req.match_info().get("fetch_mode").map(|s| s.to_string());

        let fetch_mode_or = fetch_mode.unwrap_or("posts".to_string());

        // unwrap_or("posts".to_string());

        ready(Ok(OptionalFetchMode(fetch_mode_or)))
    }
}

pub fn validate_user_password(user: &User, user_password: &str) -> bool {
    verify(user_password, &user.password).unwrap_or(false)
}
