use diesel::{prelude::PgConnection, RunQueryDsl, SelectableHelper};

use crate::models::{Comment, NewComment};
use crate::schema::comments;

pub fn create_comment(conn: &mut PgConnection, post_id: &i32, content: &str) -> Comment {
    let new_comment = NewComment { post_id, content };

    diesel::insert_into(comments::table)
        .values(&new_comment)
        .returning(Comment::as_returning())
        .get_result(conn)
        .expect("Error saving new comment")
}
