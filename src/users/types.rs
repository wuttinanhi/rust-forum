use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::User;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct UserPublic {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub user_profile_picture_url: String,
}

pub fn user_to_user_public(user: &User) -> UserPublic {
    UserPublic {
        id: user.id,
        name: user.name.clone(),
        created_at: user.created_at,
        user_profile_picture_url: user.user_profile_picture_url.clone().unwrap_or(format!(
            "https://ui-avatars.com/api/?size=250&name={}",
            user.name
        )),
    }
}
