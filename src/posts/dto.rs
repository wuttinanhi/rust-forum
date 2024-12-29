use serde::Deserialize;

use crate::{comments::types::CommentWithUser, models::Post, users::types::UserPublic};

#[derive(Deserialize)]
pub struct CreatePostFormData {
    pub title: String,
    pub body: String,
}

#[derive(Debug)]
pub struct PostPageData {
    pub post: Post,
    pub author: UserPublic,
    pub comments: Vec<CommentWithUser>,
}
