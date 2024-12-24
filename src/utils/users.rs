use actix_session::Session;
use actix_web::error::ErrorInternalServerError;

use crate::users::{
    constants::SESSION_KEY_USER,
    types::{SessionUser, UserPublic},
};

pub fn get_session_user(session: &Session) -> Result<UserPublic, actix_web::Error> {
    if let Ok(Some(user)) = session.get::<SessionUser>(SESSION_KEY_USER) {
        Ok(UserPublic {
            id: user.id,
            name: user.name,
            created_at: user.created_at,
        })
    } else {
        Err(ErrorInternalServerError("Failed to get session user!"))
    }
}
