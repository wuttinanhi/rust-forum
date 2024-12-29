use crate::db::DbError;
use crate::models::{NewPost, Post, User};
use crate::schema::posts as schema_posts;
use crate::schema::posts::dsl::*;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};

use super::types::PostWithUser;

pub fn create_post(
    conn: &mut PgConnection,
    post_user_id: &i32,
    post_title: &str,
    post_body: &str,
) -> actix_web::Result<Post, DbError> {
    let new_post_data = NewPost {
        title: post_title,
        body: post_body,
        published: true,
        user_id: *post_user_id,
    };

    let new_post = diesel::insert_into(schema_posts::table)
        .values(&new_post_data)
        .returning(Post::as_returning())
        .get_result(conn)?;

    Ok(new_post)
}

pub fn get_post(conn: &mut PgConnection, post_id: i32) -> actix_web::Result<Post, DbError> {
    let post = posts.find(post_id).first(conn)?;

    Ok(post)
}

pub fn list_posts(conn: &mut PgConnection) -> actix_web::Result<Vec<Post>, DbError> {
    let posts_vec = posts
        .filter(deleted_at.is_null())
        .order(created_at.desc())
        .load(conn)?;

    Ok(posts_vec)
}

pub fn update_post(
    conn: &mut PgConnection,
    post_id: i32,
    post_title: &str,
    post_body: &str,
) -> actix_web::Result<Post, DbError> {
    let update_result = diesel::update(posts.find(post_id))
        .set((
            title.eq(post_title),
            body.eq(post_body),
            updated_at.eq(diesel::dsl::now),
        ))
        .returning(Post::as_returning())
        .get_result(conn)?;

    Ok(update_result)
}

pub fn delete_post(conn: &mut PgConnection, post_id: i32) -> actix_web::Result<usize, DbError> {
    let delete_result = diesel::update(posts.find(post_id))
        .set(deleted_at.eq(diesel::dsl::now))
        .execute(conn)?;

    Ok(delete_result)
}

pub fn list_post_with_user(
    conn: &mut PgConnection,
) -> actix_web::Result<Vec<PostWithUser>, DbError> {
    use crate::schema::posts::dsl::{created_at, posts};
    use crate::schema::users::dsl::*;

    let posts_raw = posts
        .inner_join(users)
        .order(created_at.desc())
        .select((Post::as_select(), User::as_select()))
        .load::<(Post, User)>(conn)?;

    let posts_mapped = posts_raw
        .into_iter()
        .map(|(post, user)| PostWithUser { post, user })
        .collect();

    Ok(posts_mapped)
}
