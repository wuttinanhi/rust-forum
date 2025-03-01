use std::sync::Arc;

use diesel::{
    query_dsl::methods::FilterDsl,
    r2d2::{ConnectionManager, Pool},
    ExpressionMethods, PgConnection, RunQueryDsl, SelectableHelper,
};

use crate::{
    db::WebError,
    models::{NewPasswordReset, PasswordReset},
    utils::token::generate_random_token,
};

/// Repository trait for managing password reset tokens
pub trait TokenRepository: Send + Sync + 'static {
    /// Deletes all password reset records for a specific user
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user whose reset records should be deleted
    fn delete_password_reset_records_for_user(&self, user_id: i32) -> Result<(), WebError>;

    /// Retrieves a password reset record by its token
    ///
    /// # Arguments
    /// * `target_token` - The token string to search for
    fn get_password_reset_record(&self, target_token: &str) -> Result<PasswordReset, WebError>;

    /// Deletes a specific password reset record
    ///
    /// # Arguments
    /// * `password_reset_id` - The ID of the password reset record to delete
    fn delete_password_reset_record(&self, password_reset_id: i32) -> Result<(), WebError>;

    /// Creates a new password reset token for a user
    ///
    /// # Arguments
    /// * `target_user_id` - The ID of the user requesting password reset
    fn create_password_reset_token(&self, target_user_id: i32) -> Result<PasswordReset, WebError>;
}

pub struct PostgresTokenRepository {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

impl PostgresTokenRepository {
    pub fn new(pool: Arc<Pool<ConnectionManager<PgConnection>>>) -> Self {
        Self { pool }
    }
}

impl TokenRepository for PostgresTokenRepository {
    fn create_password_reset_token(&self, target_user_id: i32) -> Result<PasswordReset, WebError> {
        let mut conn = self.pool.get()?;

        use crate::schema::password_resets::dsl::*;

        // expire within 1 hour
        let expire_time = chrono::Utc::now().naive_utc() + chrono::Duration::hours(1);

        // generate new token
        let token = generate_random_token(16);

        let new_password_reset = NewPasswordReset {
            user_id: target_user_id,
            expires_at: expire_time,
            reset_token: token,
        };

        let password_reset = diesel::insert_into(password_resets)
            .values(&new_password_reset)
            .returning(PasswordReset::as_returning())
            .get_result(&mut conn)?;

        Ok(password_reset)
    }

    fn get_password_reset_record(&self, target_token: &str) -> Result<PasswordReset, WebError> {
        let mut conn = self.pool.get()?;

        use crate::schema::password_resets::dsl::*;

        let password_reset = password_resets
            .filter(reset_token.eq(target_token))
            .first(&mut conn)?;

        Ok(password_reset)
    }

    fn delete_password_reset_record(&self, password_reset_id: i32) -> Result<(), WebError> {
        let mut conn = self.pool.get()?;

        use crate::schema::password_resets::dsl::*;

        diesel::delete(password_resets.filter(id.eq(password_reset_id))).execute(&mut conn)?;
        Ok(())
    }

    fn delete_password_reset_records_for_user(&self, target_user_id: i32) -> Result<(), WebError> {
        let mut conn = self.pool.get()?;

        use crate::schema::password_resets::dsl::*;

        diesel::delete(password_resets.filter(user_id.eq(target_user_id))).execute(&mut conn)?;

        Ok(())
    }
}
