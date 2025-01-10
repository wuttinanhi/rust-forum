use crate::db::DbError;
use crate::models::{NewPost, Post, User};
use crate::schema::posts as schema_posts;
use crate::schema::posts::dsl::*;
use crate::utils::pagination::QueryPagination;
use crate::utils::time::time_to_human_readable;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};

use super::types::{ListPostResult, PostPublic};

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

pub fn get_posts(
    conn: &mut PgConnection,
    pagination_opts: &QueryPagination,
) -> actix_web::Result<Vec<Post>, DbError> {
    let offset_value = (pagination_opts.page - 1) * pagination_opts.limit;

    let posts_vec = posts
        .filter(deleted_at.is_null())
        .order(created_at.desc())
        .limit(pagination_opts.limit)
        .offset(offset_value)
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

pub fn get_posts_with_user(
    conn: &mut PgConnection,
    pagination_opts: &QueryPagination,
) -> actix_web::Result<ListPostResult, DbError> {
    let offset_value = (pagination_opts.page - 1) * pagination_opts.limit;

    use crate::schema::posts::dsl::{created_at, posts};
    use crate::schema::users::dsl::users;

    let posts_raw = posts
        .inner_join(users)
        .filter(deleted_at.is_null())
        .order(created_at.desc())
        .limit(pagination_opts.limit)
        .offset(offset_value)
        .select((Post::as_select(), User::as_select()))
        .load::<(Post, User)>(conn)?;

    let posts_mapped = posts_raw
        .into_iter()
        .map(|(post, user)| PostPublic {
            user,
            time_human: time_to_human_readable(post.created_at),
            post,
        })
        .collect();

    let total_posts = schema_posts::table
        .filter(deleted_at.is_null())
        .count()
        .get_result::<i64>(conn)?;

    Ok(ListPostResult {
        posts: posts_mapped,
        total: total_posts,
    })
}

pub fn get_posts_by_user(
    conn: &mut PgConnection,
    target_user_id: i32,
    pagination_opts: &QueryPagination,
) -> actix_web::Result<ListPostResult, DbError> {
    let offset_value = (pagination_opts.page - 1) * pagination_opts.limit;

    use crate::schema::posts::dsl::{created_at, deleted_at, posts, user_id};
    use crate::schema::users::dsl::users;

    let posts_raw = posts
        .inner_join(users)
        .filter(user_id.eq(target_user_id))
        .filter(deleted_at.is_null())
        .order(created_at.desc())
        .limit(pagination_opts.limit)
        .offset(offset_value)
        .select((Post::as_select(), User::as_select()))
        .load::<(Post, User)>(conn)?;

    let posts_mapped = posts_raw
        .into_iter()
        .map(|(post, user)| PostPublic {
            user,
            time_human: time_to_human_readable(post.created_at),
            post,
        })
        .collect();

    let total_posts = schema_posts::table
        .filter(user_id.eq(target_user_id))
        .filter(deleted_at.is_null())
        .count()
        .get_result::<i64>(conn)?;

    Ok(ListPostResult {
        posts: posts_mapped,
        total: total_posts,
    })
}

pub fn get_post_with_user(
    conn: &mut PgConnection,
    post_id: i32,
) -> actix_web::Result<PostPublic, DbError> {
    use crate::schema::posts as schema_posts;
    use crate::schema::users as schema_users;

    let (post, user) = posts
        .inner_join(schema_users::table)
        .filter(schema_posts::id.eq(post_id))
        .first::<(Post, User)>(conn)?;

    let post_public = PostPublic {
        user,
        time_human: time_to_human_readable(post.created_at),
        post,
    };

    Ok(post_public)
}
