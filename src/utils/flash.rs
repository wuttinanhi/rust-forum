use actix_session::{Session, SessionInsertError};
use serde_json::{json, Value};

pub fn handle_flash_message(data: &mut Value, session: &Session) {
    handle_session_flash_internal(data, session, "success".to_string());
    handle_session_flash_internal(data, session, "error".to_string());
}

fn handle_session_flash_internal(data: &mut Value, session: &Session, session_key: String) {
    let session_flash_success = session.remove_as::<String>(&session_key);

    match session_flash_success {
        Some(result) => match result {
            Ok(success_message) => {
                data.as_object_mut()
                    .unwrap()
                    .insert(session_key, json!(success_message));
            }
            // unable to deserialize session value
            Err(_json_raw) => (),
        },
        // no key present
        None => (),
    }
}

pub fn set_flash_message(
    session: &Session,
    key: &str,
    message: &str,
) -> Result<(), SessionInsertError> {
    session.insert(key, message.to_string())
}
