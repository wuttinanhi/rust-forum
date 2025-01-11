use diesel::Queryable;
use serde::{Deserialize, Serialize};

use crate::models::{Post, User};

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PostPublic {
    pub post: Post,
    pub user: User,
    pub time_human: String,
    pub allow_update: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ListPostResult {
    pub posts: Vec<PostPublic>,
    pub total: i64,
}
