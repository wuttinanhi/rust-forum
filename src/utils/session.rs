use actix_session::Session;
use serde_json::Value;

use crate::users::{constants::SESSION_KEY_USER, types::UserPublic};

pub fn handlebars_add_user(session: &Session, data: &mut Value) {
    if let Ok(Some(user)) = session.get::<UserPublic>(SESSION_KEY_USER) {
        data.as_object_mut()
            .unwrap()
            .insert("user".to_string(), serde_json::to_value(user).unwrap());
    }
}
