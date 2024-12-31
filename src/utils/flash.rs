use actix_session::{Session, SessionInsertError};
use serde_json::{json, Value};

pub const FLASH_SUCCESS: &str = "success";
pub const FLASH_WARNING: &str = "warning";
pub const FLASH_ERROR: &str = "error";

pub fn handle_flash_message(data: &mut Value, session: &Session) {
    handle_session_flash_internal(data, session, FLASH_SUCCESS.to_string());
    handle_session_flash_internal(data, session, FLASH_ERROR.to_string());
}

fn handle_session_flash_internal(data: &mut Value, session: &Session, session_key: String) {
    let session_flash_success = session.remove_as::<String>(&session_key);

    if let Some(result) = session_flash_success {
        match result {
            Ok(success_message) => {
                data.as_object_mut()
                    .unwrap()
                    .insert(session_key, json!(success_message));
            }
            // unable to deserialize session value
            Err(_json_raw) => (),
        }
    }
}

pub fn set_flash_message(
    session: &Session,
    key: &str,
    message: &str,
) -> Result<(), SessionInsertError> {
    session.insert(key, message.to_string())
}
