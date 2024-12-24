use super::types::UserPublic;
use crate::models::{NewUser, User};
use crate::schema::users as schema_users;
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};

pub fn create_user(
    conn: &mut PgConnection,
    name: &str,
    email: &str,
    password: &str,
) -> Result<User, diesel::result::Error> {
    let hashed = hash(password, DEFAULT_COST).unwrap();

    let new_user = NewUser {
        email,
        name,
        password: &hashed,
    };

    diesel::insert_into(schema_users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(conn)
    // .expect("Error saving new user")
}

pub fn login_user(
    conn: &mut PgConnection,
    user_email: &str,
    user_password: &str,
) -> Result<User, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    // Query the user by email
    let user: User = users.filter(email.eq(user_email)).first(conn)?;

    let valid = verify(user_password, &user.password).unwrap_or(false);

    // Verify the password (you would replace this with actual hashing logic)
    if valid {
        Ok(user) // Return the user if passwords match
    } else {
        Err(diesel::result::Error::NotFound) // Return NotFound for incorrect passwords
    }
}

pub fn get_user(conn: &mut PgConnection, target_user_id: i32) -> Option<User> {
    use crate::schema::users::dsl::*;
    users.filter(id.eq(target_user_id)).first(conn).ok()
}

pub fn get_user_sanitized(conn: &mut PgConnection, target_user_id: i32) -> Option<UserPublic> {
    let non_sanitized_user = get_user(conn, target_user_id)?;

    Some(UserPublic {
        id: non_sanitized_user.id,
        name: non_sanitized_user.name,
        created_at: non_sanitized_user.created_at,
    })
}
