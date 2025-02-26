use diesel::Queryable;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::{Comment, User};

#[derive(Deserialize, Validate)]
pub struct CreateCommentFormData {
    #[validate(range(min = 1, max = 1000000000, message = "Invalid post ID"))]
    pub post_id: i32,

    #[validate(length(
        min = 1,
        max = 5000,
        message = "Comment must be at least 1 character and max 5000"
    ))]
    pub body: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateCommentFormData {
    #[validate(length(
        min = 1,
        max = 5000,
        message = "Comment must be at least 1 character and max 5000"
    ))]
    pub body: String,
}

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CommentPublic {
    pub comment: Comment,
    pub user: User,
    pub time_human: String,
    pub allow_update: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ListCommentResult {
    pub comments: Vec<CommentPublic>,
    pub total: i64,
}
