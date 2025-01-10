use actix_session::Session;
use serde_json::Value;

use super::users::get_session_user;

pub fn handlebars_add_user(session: &Session, data: &mut Value) -> Result<(), actix_web::Error> {
    let session_user = get_session_user(&session)?;

    data.as_object_mut().unwrap().insert(
        "user".to_string(),
        serde_json::to_value(session_user).unwrap(),
    );

    Ok(())
}
