use std::sync::Arc;

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use diesel::{
    query_dsl::methods::FilterDsl,
    r2d2::{ConnectionManager, Pool},
    ExpressionMethods, PgConnection, RunQueryDsl, SelectableHelper,
};

use crate::{
    db::WebError,
    entities::user::{user_to_user_public, UserPublic},
    models::{NewPasswordReset, NewUser, PasswordReset, UpdateUserNameAndProfilePicture, User},
    utils::token::generate_random_token,
};

pub trait UserRepository: Send + Sync {
    type Error;

    fn create_user(&self, name: &str, email: &str, password: &str) -> Result<User, Self::Error>;
    fn login_user(&self, email: &str, password: &str) -> Result<User, Self::Error>;
    fn get_user_by_id(&self, user_id: i32) -> Result<User, Self::Error>;
    fn get_user_by_email(&self, email: &str) -> Result<User, Self::Error>;
    fn get_user_sanitized_by_id(&self, user_id: i32) -> Result<UserPublic, Self::Error>;
    fn update_user_password(&self, user: &User, new_password: &str) -> Result<(), Self::Error>;
    fn update_user_email(&self, user: &User, new_email: &str) -> Result<(), Self::Error>;
    fn update_user_data(
        &self,
        user: &User,
        new_data: &UpdateUserNameAndProfilePicture,
    ) -> Result<(), Self::Error>;
    fn delete_user(&self, user: &User) -> Result<(), Self::Error>;
    fn create_password_reset(&self, user: &User) -> Result<PasswordReset, Self::Error>;
    fn get_password_reset_by_token(&self, token: &str) -> Result<PasswordReset, Self::Error>;
    fn delete_password_reset(&self, password_reset: &PasswordReset) -> Result<(), Self::Error>;
    fn delete_password_resets_for_user(&self, user: &User) -> Result<(), Self::Error>;
    fn update_user_password_from_reset(
        &self,
        password_reset: &PasswordReset,
        new_password: &str,
    ) -> Result<(), Self::Error>;
}

pub type UserRepositoryWithError = dyn UserRepository<Error = WebError>;

pub struct PostgresUserRepository {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

impl PostgresUserRepository {
    pub fn new(pool: Arc<Pool<ConnectionManager<PgConnection>>>) -> Self {
        Self { pool }
    }

    fn validate_user_password(&self, user: &User, user_password: &str) -> bool {
        verify(user_password, &user.password).unwrap_or(false)
    }
}

impl UserRepository for PostgresUserRepository {
    type Error = WebError;

    fn create_user(&self, name: &str, email: &str, password: &str) -> Result<User, WebError> {
        use crate::schema::users::table as users_table;

        let mut conn = self.pool.get()?;

        let hashed = hash(password, DEFAULT_COST).unwrap();

        let new_user_data = NewUser {
            email,
            name,
            password: &hashed,
        };

        let new_user = diesel::insert_into(users_table)
            .values(&new_user_data)
            .returning(User::as_returning())
            .get_result(&mut conn)?;

        Ok(new_user)
    }

    fn login_user(&self, user_email: &str, user_password: &str) -> Result<User, WebError> {
        // Query the user by email
        let user: User = self.get_user_by_email(user_email)?;

        let valid = self.validate_user_password(&user, user_password);

        // Verify the password (you would replace this with actual hashing logic)
        if valid {
            // Return the user if passwords match
            Ok(user)
        } else {
            // Otherwise, return an error
            Err(Box::new(diesel::result::Error::NotFound))
        }
    }

    fn get_user_by_id(&self, user_id: i32) -> Result<User, WebError> {
        let mut conn = self.pool.get()?;

        use crate::schema::users::dsl::*;

        let user = users.filter(id.eq(user_id)).first(&mut conn)?;
        Ok(user)
    }

    fn get_user_by_email(&self, user_email: &str) -> Result<User, WebError> {
        let mut conn = self.pool.get()?;

        use crate::schema::users::dsl::*;

        let user = users.filter(email.eq(user_email)).first(&mut conn)?;
        Ok(user)
    }

    fn get_user_sanitized_by_id(&self, target_user_id: i32) -> Result<UserPublic, WebError> {
        let non_sanitized_user = self.get_user_by_id(target_user_id)?;

        let user_public = user_to_user_public(&non_sanitized_user);

        Ok(user_public)
    }

    fn update_user_password(&self, user: &User, new_password: &str) -> Result<(), WebError> {
        let mut conn = self.pool.get()?;

        use crate::schema::users::dsl::*;

        let hashed = hash(new_password, DEFAULT_COST).unwrap();

        diesel::update(users.filter(id.eq(user.id)))
            .set(password.eq(&hashed))
            .execute(&mut conn)?;

        Ok(())
    }

    fn update_user_email(&self, user: &User, new_email: &str) -> Result<(), WebError> {
        let mut conn = self.pool.get()?;

        use crate::schema::users::dsl::*;

        diesel::update(users.filter(id.eq(user.id)))
            .set(email.eq(new_email))
            .execute(&mut conn)?;
        Ok(())
    }

    fn update_user_data(
        &self,
        user: &User,
        new_data: &UpdateUserNameAndProfilePicture,
    ) -> Result<(), WebError> {
        let mut conn = self.pool.get()?;

        use crate::schema::users::dsl::*;

        diesel::update(users.filter(id.eq(user.id)))
            .set(new_data)
            .execute(&mut conn)?;

        Ok(())
    }

    fn delete_user(&self, user: &User) -> Result<(), WebError> {
        let mut conn = self.pool.get()?;

        use crate::schema::users::dsl::*;

        diesel::delete(users.filter(id.eq(user.id))).execute(&mut conn)?;
        Ok(())
    }

    fn create_password_reset(&self, user: &User) -> Result<PasswordReset, WebError> {
        let mut conn = self.pool.get()?;

        use crate::schema::password_resets::dsl::*;

        // expire within 1 hour
        let expire_time = chrono::Utc::now().naive_utc() + Duration::hours(1);
        let token = generate_random_token(16);

        let new_password_reset = NewPasswordReset {
            user_id: user.id,
            expires_at: expire_time,
            reset_token: token,
        };

        let password_reset = diesel::insert_into(password_resets)
            .values(&new_password_reset)
            .returning(PasswordReset::as_returning())
            .get_result(&mut conn)?;

        Ok(password_reset)
    }

    fn get_password_reset_by_token(&self, target_token: &str) -> Result<PasswordReset, WebError> {
        let mut conn = self.pool.get()?;

        use crate::schema::password_resets::dsl::*;

        let password_reset = password_resets
            .filter(reset_token.eq(target_token))
            .first(&mut conn)?;

        Ok(password_reset)
    }

    fn delete_password_reset(&self, password_reset: &PasswordReset) -> Result<(), WebError> {
        let mut conn = self.pool.get()?;

        use crate::schema::password_resets::dsl::*;

        diesel::delete(password_resets.filter(id.eq(password_reset.id))).execute(&mut conn)?;
        Ok(())
    }

    fn delete_password_resets_for_user(&self, user: &User) -> Result<(), WebError> {
        let mut conn = self.pool.get()?;

        use crate::schema::password_resets::dsl::*;

        diesel::delete(password_resets.filter(user_id.eq(user.id))).execute(&mut conn)?;
        Ok(())
    }

    fn update_user_password_from_reset(
        &self,
        password_reset: &PasswordReset,
        new_password: &str,
    ) -> Result<(), WebError> {
        if password_reset.expires_at < Utc::now().naive_utc() {
            return Err(WebError::from("password reset expired"));
        }

        let user = self.get_user_by_id(password_reset.user_id)?;

        self.update_user_password(&user, new_password)?;

        self.delete_password_reset(password_reset)?;

        Ok(())
    }
}
