use actix_session::Session;

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
        Err(actix_web::error::ErrorUnauthorized("Unauthorized"))
    }
}

#[macro_export]
macro_rules! validate_password_and_confirm_password {
    ($form:expr) => {{
        let new_password_and_confirm_password_equal = $form.new_password == $form.confirm_password;
        if !new_password_and_confirm_password_equal {
            return Err(actix_web::error::ErrorBadRequest(
                "New password and confirm password not equal!",
            ));
        }
    }};
}
