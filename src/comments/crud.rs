use crate::models::{Comment, NewComment};
use crate::schema::comments as schema_comments;
use crate::schema::comments::dsl::*;
use crate::schema::comments::post_id;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};

pub fn create_comment(
    conn: &mut PgConnection,
    comment_user_id: &i32,
    parent_post_id: &i32,
    comment_body: &str,
) -> Comment {
    let new_comment = NewComment {
        post_id: *parent_post_id,
        user_id: *comment_user_id,
        content: comment_body,
    };

    diesel::insert_into(schema_comments::table)
        .values(&new_comment)
        .returning(Comment::as_returning())
        .get_result(conn)
        .expect("Error saving new comment")
}

pub fn get_comment(conn: &mut PgConnection, comment_id: &i32) -> Option<Comment> {
    comments.find(comment_id).first(conn).ok()
}

pub fn list_comments(conn: &mut PgConnection, parent_post_id: &i32) -> Vec<Comment> {
    comments
        .filter(post_id.eq(parent_post_id))
        .order(created_at.desc())
        .load(conn)
        .expect("Error loading comments")
}

pub fn update_comment(
    conn: &mut PgConnection,
    target_comment_id: &i32,
    new_body: &str,
) -> Option<Comment> {
    diesel::update(comments.find(target_comment_id))
        .set(content.eq(new_body))
        .returning(Comment::as_returning())
        .get_result(conn)
        .ok()
}

pub fn delete_post(conn: &mut PgConnection, target_post_id: i32) -> bool {
    diesel::update(comments.find(target_post_id))
        .set(deleted_at.eq(diesel::dsl::now))
        .execute(conn)
        .is_ok()
}
