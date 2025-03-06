use crate::{
    db::WebError,
    entities::user::UserPublic,
    models::{UpdateUserNameAndProfilePicture, User},
};

/// Trait defining the interface for user-related operations in the repository.
/// Requires implementation of Send + Sync for thread safety.
pub trait UserRepository: Send + Sync + 'static {
    /// The error type that will be returned by operations in this repository
    type Error;

    /// Creates a new user with the given name, email, and password
    fn create_user(&self, name: &str, email: &str, password: &str) -> Result<User, Self::Error>;

    /// Authenticates a user with email and password
    fn login_user(&self, email: &str, password: &str) -> Result<User, Self::Error>;

    /// Retrieves a user by their ID
    fn get_user_by_id(&self, user_id: i32) -> Result<User, Self::Error>;

    /// Retrieves a user by their email address
    fn get_user_by_email(&self, email: &str) -> Result<User, Self::Error>;

    /// Gets a sanitized (public) version of a user by their ID
    fn get_user_sanitized_by_id(&self, user_id: i32) -> Result<UserPublic, Self::Error>;

    /// Updates a user's password
    fn update_user_password(&self, user: &User, new_password: &str) -> Result<(), Self::Error>;

    /// Updates a user's email address
    fn update_user_email(&self, user: &User, new_email: &str) -> Result<(), Self::Error>;

    /// Updates a user's name and profile picture
    fn update_user_data(
        &self,
        user: &User,
        new_data: &UpdateUserNameAndProfilePicture,
    ) -> Result<(), Self::Error>;

    /// Deletes a user from the system
    fn delete_user(&self, user: &User) -> Result<(), Self::Error>;
}

pub type UserRepositoryWithError = dyn UserRepository<Error = WebError>;
