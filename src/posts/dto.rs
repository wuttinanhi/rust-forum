use serde::Deserialize;

use crate::{comments::types::CommentPublic, models::Post, users::types::UserPublic};

#[derive(Deserialize)]
pub struct CreatePostFormData {
    pub title: String,
    pub body: String,
}
