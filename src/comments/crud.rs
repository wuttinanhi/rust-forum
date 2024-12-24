use diesel::{prelude::PgConnection, RunQueryDsl, SelectableHelper};

use crate::models::{Comment, NewComment};
use crate::schema::comments;

pub fn create_comment(
    conn: &mut PgConnection,
    comment_user_id: &i32,
    post_id: &i32,
    content: &str,
) -> Comment {
    let new_comment = NewComment {
        post_id: *post_id,
        user_id: *comment_user_id,
        content,
    };

    diesel::insert_into(comments::table)
        .values(&new_comment)
        .returning(Comment::as_returning())
        .get_result(conn)
        .expect("Error saving new comment")
}
