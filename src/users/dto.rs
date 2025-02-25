use std::sync::LazyLock;

use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use serde::Deserialize;
use validator::Validate;

use regex::Regex;

static NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-zA-Z\s]+$").unwrap());

#[derive(Deserialize, Debug, Validate)]
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

#[derive(Deserialize, Debug, Validate)]
pub struct UserRegisterFormData {
    #[validate(regex(
        path = *NAME_REGEX,
        message = "Name must contain only letters and spaces"
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

#[derive(Deserialize, Debug, Validate)]
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

#[derive(Deserialize, Debug, Validate)]
pub struct UserUpdateFormData {
    #[validate(regex(
        path = *NAME_REGEX,
        message = "Name must contain only letters and spaces"
    ))]
    pub new_name: String,
}

#[derive(Debug, MultipartForm)]
pub struct UserUploadProfilePictureForm {
    #[multipart(limit = "10MB")]
    pub profile_picture: TempFile,
}

#[derive(Deserialize, Debug, Validate)]
pub struct UserPasswordResetRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct UserPasswordResetTokenQueryString {
    #[validate(length(min = 10, max = 30, message = "Invalid token"))]
    pub token: String,
}

#[derive(Deserialize, Debug, Validate)]
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
