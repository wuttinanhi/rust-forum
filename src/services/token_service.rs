use std::{fmt::Display, sync::Arc};

use crate::{models::PasswordReset, repositories::token_repository::TokenRepository};

pub trait TokenService: Send + Sync {
    fn create_password_reset(
        &self,
        target_user_id: i32,
    ) -> Result<PasswordReset, TokenServiceError>;
    fn get_password_reset_by_token(
        &self,
        target_token: &str,
    ) -> Result<PasswordReset, TokenServiceError>;
    fn delete_password_reset(&self, user_id: i32) -> Result<usize, TokenServiceError>;
    fn delete_password_resets_by_user(&self, user_id: i32) -> Result<usize, TokenServiceError>;
}

pub enum TokenServiceError {
    ErrorCreate(String),
    ErrorGet(String),
    ErrorDelete(String),
}

impl Display for TokenServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenServiceError::ErrorCreate(msg) => write!(f, "Failed to create token: {}", msg),
            TokenServiceError::ErrorGet(msg) => write!(f, "Failed to get token: {}", msg),
            TokenServiceError::ErrorDelete(msg) => write!(f, "Failed to delete token: {}", msg),
        }
    }
}

pub struct BasedTokenService {
    token_repository: Arc<dyn TokenRepository>,
}

impl BasedTokenService {
    pub fn new(token_repository: Arc<dyn TokenRepository>) -> Self {
        Self { token_repository }
    }
}

impl TokenService for BasedTokenService {
    fn create_password_reset(
        &self,
        target_user_id: i32,
    ) -> Result<PasswordReset, TokenServiceError> {
        self.token_repository
            .create_password_reset(target_user_id)
            .map_err(|e| TokenServiceError::ErrorCreate(e.to_string()))
    }

    fn get_password_reset_by_token(
        &self,
        target_token: &str,
    ) -> Result<PasswordReset, TokenServiceError> {
        self.token_repository
            .get_password_reset(target_token)
            .map_err(|e| TokenServiceError::ErrorGet(e.to_string()))
    }

    fn delete_password_reset(&self, user_id: i32) -> Result<usize, TokenServiceError> {
        self.token_repository
            .delete_password_reset(user_id)
            .map_err(|e| TokenServiceError::ErrorDelete(e.to_string()))
    }

    fn delete_password_resets_by_user(&self, user_id: i32) -> Result<usize, TokenServiceError> {
        self.token_repository
            .delete_password_resets_for_user(user_id)
            .map_err(|e| TokenServiceError::ErrorDelete(e.to_string()))
    }
}
