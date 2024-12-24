use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::User;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct UserPublic {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
}

pub fn user_to_user_public(user: &User) -> UserPublic {
    UserPublic {
        id: user.id,
        name: user.name.clone(),
        created_at: user.created_at,
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SessionUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

pub fn user_to_user_session(user: &User) -> SessionUser {
    SessionUser {
        id: user.id,
        name: user.name.clone(),
        email: user.email.clone(),
        created_at: user.created_at,
        updated_at: user.updated_at,
    }
}
