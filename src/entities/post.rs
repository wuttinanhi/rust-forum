use crate::models::{Post, User};
use diesel::Queryable;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct PostFormData {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Title must be at least 1 character and max 50"
    ))]
    pub title: String,

    #[validate(length(
        min = 1,
        max = 5000,
        message = "Body must be at least 1 character and max 5000"
    ))]
    pub body: String,
}

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
