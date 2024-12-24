use diesel::Queryable;
use serde::{Deserialize, Serialize};

use crate::models::{Post, User};

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PostWithUser {
    pub post: Post,
    pub user: User,
}
