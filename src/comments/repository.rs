use crate::db::DbError;
use crate::models::{Comment, NewComment, User};
use crate::schema::comments as schema_comments;
use crate::schema::users::dsl::*;
use crate::utils::pagination::QueryPagination;
use crate::utils::time::time_to_human_readable;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};

use super::types::{CommentPublic, ListCommentResult};

pub fn create_comment(
    conn: &mut PgConnection,
    comment_user_id: &i32,
    parent_post_id: &i32,
    comment_body: &str,
) -> actix_web::Result<Comment, DbError> {
    let new_comment_data = NewComment {
        post_id: *parent_post_id,
        user_id: *comment_user_id,
        content: comment_body,
    };

    let new_comment = diesel::insert_into(schema_comments::table)
        .values(&new_comment_data)
        .returning(Comment::as_returning())
        .get_result(conn)?;

    Ok(new_comment)
}

pub fn get_comment(
    conn: &mut PgConnection,
    comment_id: &i32,
) -> actix_web::Result<Comment, DbError> {
    use crate::schema::comments::dsl::*;
    let comment = comments.find(comment_id).first(conn)?;
    Ok(comment)
}

pub fn get_comments(
    conn: &mut PgConnection,
    parent_post_id: &i32,
) -> actix_web::Result<Vec<Comment>, DbError> {
    use crate::schema::comments::dsl::*;

    let comments_vec = comments
        .filter(post_id.eq(parent_post_id))
        .order(created_at.desc())
        .load(conn)?;

    Ok(comments_vec)
}

pub fn update_comment(
    conn: &mut PgConnection,
    target_comment_id: &i32,
    new_body: &str,
) -> actix_web::Result<Comment, DbError> {
    use crate::schema::comments::dsl::*;

    let comment = diesel::update(comments.find(target_comment_id))
        .set(content.eq(new_body))
        .returning(Comment::as_returning())
        .get_result(conn)?;

    Ok(comment)
}

pub fn delete_comment(
    conn: &mut PgConnection,
    target_post_id: i32,
) -> actix_web::Result<usize, DbError> {
    use crate::schema::comments::dsl::*;

    let delete_usize = diesel::update(comments.find(target_post_id))
        .set(deleted_at.eq(diesel::dsl::now))
        .execute(conn)?;

    Ok(delete_usize)
}

pub fn get_comments_with_user(
    conn: &mut PgConnection,
    parent_post_id: &i32,
) -> actix_web::Result<Vec<CommentPublic>, DbError> {
    use crate::schema::comments::dsl::{comments, deleted_at, post_id};

    let comments_joined = comments
        .inner_join(users)
        .filter(post_id.eq(parent_post_id))
        .filter(deleted_at.is_null())
        .order(created_at.asc())
        .select((Comment::as_select(), User::as_select()))
        .load::<(Comment, User)>(conn)?;

    let comments_mapped = comments_joined
        .into_iter()
        .map(|(comment, user)| CommentPublic {
            time_human: time_to_human_readable(comment.created_at),
            comment,
            user,
        })
        .collect();

    Ok(comments_mapped)
}

pub fn get_comments_by_user(
    conn: &mut PgConnection,
    target_user_id: &i32,
    pagination_opts: &QueryPagination,
) -> actix_web::Result<ListCommentResult, DbError> {
    let offset_value = (pagination_opts.page - 1) * pagination_opts.limit;

    use crate::schema::comments::dsl::{comments, deleted_at, user_id};
    use crate::schema::users::dsl::users;

    let comments_joined = comments
        .inner_join(users)
        .filter(user_id.eq(target_user_id))
        .filter(deleted_at.is_null())
        .order(created_at.desc())
        .limit(pagination_opts.limit)
        .offset(offset_value)
        .select((Comment::as_select(), User::as_select()))
        .load::<(Comment, User)>(conn)?;

    let comments_mapped = comments_joined
        .into_iter()
        .map(|(comment, user)| CommentPublic {
            time_human: time_to_human_readable(comment.created_at),
            comment,
            user,
        })
        .collect();

    let total = schema_comments::table
        .filter(user_id.eq(target_user_id))
        .filter(deleted_at.is_null())
        .count()
        .get_result::<i64>(conn)?;

    Ok(ListCommentResult {
        comments: comments_mapped,
        total,
    })
}
