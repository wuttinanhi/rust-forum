use actix_session::Session;
use actix_web::error::ErrorInternalServerError;

use crate::users::{constants::SESSION_KEY_USER, types::UserPublic};

pub fn get_session_user(session: &Session) -> Result<UserPublic, actix_web::Error> {
    if let Ok(Some(user)) = session.get::<UserPublic>(SESSION_KEY_USER) {
        Ok(UserPublic {
            id: user.id,
            name: user.name,
            created_at: user.created_at,
            user_profile_picture_url: user.user_profile_picture_url,
        })
    } else {
        Err(ErrorInternalServerError("Failed to get session user!"))
    }
}

#[macro_export]
macro_rules! validate_input_user_name {
    ($name:expr) => {
        if $name.len() < 3 || $name.len() > 15 {
            return Err(actix_web::error::ErrorBadRequest(
                "Invalid username length! (name must be between 3 and 15 characters)",
            ));
        }
    };
}

#[macro_export]
macro_rules! validate_input_user_password {
    ($password:expr) => {
        if $password.len() < 3 || $password.len() > 128 {
            return Err(actix_web::error::ErrorBadRequest(
                "Invalid password length! (password must be between 3 and 128 characters)",
            ));
        }
    };
}
