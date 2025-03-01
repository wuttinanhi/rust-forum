use crate::{db::WebError, models::PasswordReset};

pub trait TokenService: Send + Sync {
    fn create_password_reset(&self, target_user_id: i32) -> Result<PasswordReset, WebError>;
    fn get_password_reset_by_token(&self, target_token: &str) -> Result<PasswordReset, WebError>;
}

pub struct BasedTokenService {}

impl Default for BasedTokenService {
    fn default() -> Self {
        Self::new()
    }
}

impl BasedTokenService {
    pub fn new() -> Self {
        Self {}
    }
}

impl TokenService for BasedTokenService {
    fn create_password_reset(&self, target_user_id: i32) -> Result<PasswordReset, WebError> {
        todo!()
    }

    fn get_password_reset_by_token(&self, target_token: &str) -> Result<PasswordReset, WebError> {
        todo!()
    }
}
