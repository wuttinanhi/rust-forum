use actix_web::{FromRequest, HttpRequest};
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

use futures::future::{ready, Ready};

pub struct OptionalFetchMode(pub String);

impl FromRequest for OptionalFetchMode {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let fetch_mode = req.match_info().get("fetch_mode").map(|s| s.to_string());

        let fetch_mode_or = fetch_mode.unwrap_or("posts".to_string());

        // unwrap_or("posts".to_string());

        ready(Ok(OptionalFetchMode(fetch_mode_or)))
    }
}
