use crate::{db::WebError, models::PasswordReset};

pub trait TokenService: Send + Sync {
    fn create_password_reset(&self, target_user_id: i32) -> Result<PasswordReset, WebError>;
    fn get_password_reset_by_token(&self, target_token: &str) -> Result<PasswordReset, WebError>;
}
