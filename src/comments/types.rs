use diesel::Queryable;
use serde::{Deserialize, Serialize};

use crate::models::{Comment, User};

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CommentWithUser {
    pub comment: Comment,
    pub user: User,
}
