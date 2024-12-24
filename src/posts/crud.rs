use crate::models::{NewPost, Post};
use crate::schema::posts as post_schema;
use crate::schema::posts::dsl::*;
use diesel::prelude::*;
use diesel::{prelude::PgConnection, RunQueryDsl, SelectableHelper};

pub fn create_post(
    conn: &mut PgConnection,
    post_user_id: &i32,
    post_title: &str,
    post_body: &str,
) -> Post {
    let new_post = NewPost {
        title: post_title,
        body: post_body,
        published: true,
        user_id: *post_user_id,
    };

    diesel::insert_into(post_schema::table)
        .values(&new_post)
        .returning(Post::as_returning())
        .get_result(conn)
        .expect("Error saving new post")
}

pub fn get_post(conn: &mut PgConnection, post_id: i32) -> Option<Post> {
    posts.find(post_id).first(conn).ok()
}

pub fn list_posts(conn: &mut PgConnection) -> Vec<Post> {
    posts
        .filter(deleted_at.is_null())
        .order(created_at.desc())
        .load(conn)
        .expect("Error loading posts")
}

pub fn update_post(
    conn: &mut PgConnection,
    post_id: i32,
    post_title: &str,
    post_body: &str,
) -> Option<Post> {
    diesel::update(posts.find(post_id))
        .set((
            title.eq(post_title),
            body.eq(post_body),
            updated_at.eq(diesel::dsl::now),
        ))
        .returning(Post::as_returning())
        .get_result(conn)
        .ok()
}

pub fn delete_post(conn: &mut PgConnection, post_id: i32) -> bool {
    use crate::schema::posts::dsl::*;

    diesel::update(posts.find(post_id))
        .set(deleted_at.eq(diesel::dsl::now))
        .execute(conn)
        .is_ok()
}
