use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct UserLoginFormData {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct UserRegisterFormData {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct UserChangePasswordFormData {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Deserialize, Debug)]
pub struct UserUpdateFormData {
    pub new_name: String,
}

#[derive(Debug, MultipartForm)]
pub struct UserUploadProfilePictureForm {
    #[multipart(limit = "10MB")]
    pub profile_picture: TempFile,
}

#[derive(Deserialize, Debug)]
pub struct UserPasswordResetRequest {
    pub email: String,
}

#[derive(Deserialize, Debug)]
pub struct UserPasswordResetTokenQueryString {
    pub token: String,
}

#[derive(Deserialize, Debug)]
pub struct UserPasswordResetTokenRequest {
    pub token: String,
    pub new_password: String,
    pub confirm_password: String,
}
