use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

use crate::{
    entities::user::{validate_user_password, UserPublic},
    models::{PasswordReset, UpdateUserNameAndProfilePicture, User},
    repositories::{token_repository::TokenRepository, user_repository::UserRepositoryWithError},
};

pub trait UserService: Send + Sync {
    fn register_user(
        &self,
        user_name: &str,
        user_email: &str,
        user_password: &str,
    ) -> Result<User, UserServiceError>;

    fn login_user(&self, user_email: &str, user_password: &str) -> Result<User, UserServiceError>;

    fn get_user_by_id(&self, user_id: i32) -> Result<User, UserServiceError>;

    fn get_user_by_id_public(&self, user_id: i32) -> Result<UserPublic, UserServiceError>;

    fn get_user_by_email(&self, email: &str) -> Result<User, UserServiceError>;

    fn update_user_data(
        &self,
        user_id: i32,
        new_data: &UpdateUserNameAndProfilePicture,
    ) -> Result<User, UserServiceError>;

    fn update_user_password(
        &self,
        user_id: i32,
        new_password: &str,
    ) -> Result<User, UserServiceError>;

    /// Updates a user's password using a password reset token
    fn update_user_password_from_reset(
        &self,
        password_reset: &PasswordReset,
        new_password: &str,
    ) -> Result<(), UserServiceError>;
}

pub struct BasedUserService {
    user_repository: Arc<UserRepositoryWithError>,
    token_repository: Arc<dyn TokenRepository>,
}

impl BasedUserService {
    pub fn new(
        user_repository: Arc<UserRepositoryWithError>,
        token_repository: Arc<dyn TokenRepository>,
    ) -> Self {
        Self {
            user_repository,
            token_repository,
        }
    }
}

pub enum UserServiceError {
    ErrorLogin,
    ErrorRegister,
    ErrorGetData(&'static str),
    ErrorChangePassword,
    ErrorUpdateUserData,
    ErrorInternal,
}

impl Display for UserServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UserServiceError::ErrorLogin => write!(f, "Login failed"),
            UserServiceError::ErrorRegister => write!(f, "Registration failed"),
            UserServiceError::ErrorGetData(msg) => write!(f, "Data retrieval error: {}", msg),
            UserServiceError::ErrorChangePassword => write!(f, "Password change failed"),
            UserServiceError::ErrorUpdateUserData => write!(f, "User data update failed"),
            UserServiceError::ErrorInternal => write!(f, "Internal server error"),
        }
    }
}

impl UserService for BasedUserService {
    fn login_user(&self, user_email: &str, user_password: &str) -> Result<User, UserServiceError> {
        // Query the user by email
        let user: User = self
            .user_repository
            .get_user_by_email(user_email)
            .map_err(|_| UserServiceError::ErrorLogin)?;

        let valid = validate_user_password(&user, user_password);

        // Verify the password
        if valid {
            // Return the user if passwords match
            Ok(user)
        } else {
            // Otherwise, return an error
            Err(UserServiceError::ErrorLogin)
        }
    }

    fn register_user(
        &self,
        user_name: &str,
        user_email: &str,
        user_password: &str,
    ) -> Result<User, UserServiceError> {
        let create_user_result = self
            .user_repository
            .create_user(user_name, user_email, user_password)
            .map_err(|e| UserServiceError::ErrorRegister)?;

        Ok(create_user_result)
    }

    fn get_user_by_id(&self, user_id: i32) -> Result<User, UserServiceError> {
        let user = self
            .user_repository
            .get_user_by_id(user_id)
            .map_err(|_| UserServiceError::ErrorGetData("failed to get user data"))?;

        Ok(user)
    }

    fn get_user_by_id_public(&self, user_id: i32) -> Result<UserPublic, UserServiceError> {
        let user_sanitized = self
            .user_repository
            .get_user_sanitized_by_id(user_id)
            .map_err(|_| UserServiceError::ErrorGetData("failed to get user data"))?;

        Ok(user_sanitized)
    }

    fn update_user_data(
        &self,
        user_id: i32,
        new_data: &UpdateUserNameAndProfilePicture,
    ) -> Result<User, UserServiceError> {
        // get user data
        let user = self.get_user_by_id(user_id)?;

        // update user data
        self.user_repository
            .update_user_data(&user, new_data)
            .map_err(|e| UserServiceError::ErrorUpdateUserData)?;

        // get user data
        let updated_user = self.get_user_by_id(user_id)?;

        Ok(updated_user)
    }

    fn update_user_password(
        &self,
        user_id: i32,
        new_password: &str,
    ) -> Result<User, UserServiceError> {
        // get user data
        let user = self.get_user_by_id(user_id)?;

        // update user password
        self.user_repository
            .update_user_password(&user, new_password)
            .map_err(|_| UserServiceError::ErrorChangePassword)?;

        Ok(user)
    }

    fn get_user_by_email(&self, user_email: &str) -> Result<User, UserServiceError> {
        let user = self
            .user_repository
            .get_user_by_email(user_email)
            .map_err(|_| UserServiceError::ErrorGetData("failed to get user data"))?;
        Ok(user)
    }

    fn update_user_password_from_reset(
        &self,
        password_reset: &PasswordReset,
        new_password: &str,
    ) -> Result<(), UserServiceError> {
        if password_reset.expires_at < chrono::Utc::now().naive_utc() {
            return Err(UserServiceError::ErrorChangePassword);
        }

        let user = self.get_user_by_id(password_reset.user_id)?;

        self.update_user_password(user.id, new_password)?;

        self.token_repository
            .delete_password_reset_records_for_user(user.id)
            .map_err(|_| {
                println!("failed to delete_password_reset_records_for_user");
                UserServiceError::ErrorInternal
            })?;

        Ok(())
    }
}
