use diesel::Queryable;
use serde::{Deserialize, Serialize};

use crate::models::{Comment, User};

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CommentPublic {
    pub comment: Comment,
    pub user: User,
    pub time_human: String,
}

pub struct ListCommentResult {
    pub comments: Vec<CommentPublic>,
    pub total: i64,
}
