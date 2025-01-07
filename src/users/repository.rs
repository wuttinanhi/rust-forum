use super::types::{user_to_user_public, UserPublic};
use crate::db::DbError;
use crate::models::{NewUser, UpdateUserNameAndProfilePicture, User};
use crate::schema::users as schema_users;
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::result::Error;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};

pub fn create_user(
    conn: &mut PgConnection,
    name: &str,
    email: &str,
    password: &str,
) -> Result<User, DbError> {
    let hashed = hash(password, DEFAULT_COST).unwrap();

    let new_user_data = NewUser {
        email,
        name,
        password: &hashed,
    };

    let new_user = diesel::insert_into(schema_users::table)
        .values(&new_user_data)
        .returning(User::as_returning())
        .get_result(conn)?;

    Ok(new_user)
}

pub fn login_user(
    conn: &mut PgConnection,
    user_email: &str,
    user_password: &str,
) -> Result<User, DbError> {
    // Query the user by email
    let user: User = get_user_by_email(conn, user_email)?;

    let valid = validate_user_password(&user, user_password);

    // Verify the password (you would replace this with actual hashing logic)
    if valid {
        // Return the user if passwords match
        Ok(user)
    } else {
        // Otherwise, return an error
        Err(Box::new(Error::NotFound))
    }
}

pub fn get_user_by_id(conn: &mut PgConnection, target_user_id: i32) -> Result<User, DbError> {
    use crate::schema::users::dsl::*;
    let user = users.filter(id.eq(target_user_id)).first(conn)?;
    Ok(user)
}

pub fn get_user_by_email(conn: &mut PgConnection, user_email: &str) -> Result<User, DbError> {
    use crate::schema::users::dsl::*;
    let user = users.filter(email.eq(user_email)).first(conn)?;
    Ok(user)
}

pub fn validate_user_password(user: &User, user_password: &str) -> bool {
    verify(user_password, &user.password).unwrap_or(false)
}

pub fn get_user_sanitized_by_id(
    conn: &mut PgConnection,
    target_user_id: i32,
) -> Result<UserPublic, DbError> {
    let non_sanitized_user = get_user_by_id(conn, target_user_id)?;

    let user_public = user_to_user_public(&non_sanitized_user);

    Ok(user_public)
}

pub fn update_user_password(
    conn: &mut PgConnection,
    user: &User,
    new_password: &str,
) -> Result<(), DbError> {
    use crate::schema::users::dsl::*;

    let hashed = hash(new_password, DEFAULT_COST).unwrap();

    diesel::update(users.filter(id.eq(user.id)))
        .set(password.eq(&hashed))
        .execute(conn)?;

    Ok(())
}

pub fn update_user_email(
    conn: &mut PgConnection,
    user: &User,
    new_email: &str,
) -> Result<(), DbError> {
    use crate::schema::users::dsl::*;
    diesel::update(users.filter(id.eq(user.id)))
        .set(email.eq(new_email))
        .execute(conn)?;
    Ok(())
}

pub fn update_user_data(
    conn: &mut PgConnection,
    user: &User,
    new_data: &UpdateUserNameAndProfilePicture,
) -> Result<(), DbError> {
    use crate::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(user.id)))
        .set(new_data)
        .execute(conn)?;
    Ok(())
}

pub fn delete_user(conn: &mut PgConnection, user: &User) -> Result<(), DbError> {
    use crate::schema::users::dsl::*;
    diesel::delete(users.filter(id.eq(user.id))).execute(conn)?;
    Ok(())
}
