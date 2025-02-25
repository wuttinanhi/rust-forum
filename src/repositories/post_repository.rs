use std::sync::Arc;

use crate::db::WebError;
use crate::models::{NewPost, Post, User};

use crate::utils::pagination::QueryPagination;
use crate::utils::time::time_to_human_readable;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::entities::post::{ListPostResult, PostPublic};
use async_trait::async_trait;

#[async_trait]
pub trait PostRepository {
    type Error;

    /// Creates a new post
    async fn create_post(
        &self,
        owner_user_id: i32,
        post_title: &str,
        post_body: &str,
    ) -> Result<Post, Self::Error>;

    /// Retrieves a post by its ID
    async fn get_post(&self, post_id: i32) -> Result<Post, Self::Error>;

    /// Retrieves a paginated list of posts
    async fn get_posts(&self, pagination: &QueryPagination) -> Result<Vec<Post>, Self::Error>;

    /// Updates an existing post
    async fn update_post(
        &self,
        post_id: i32,
        post_title: &str,
        post_body: &str,
    ) -> Result<Post, Self::Error>;

    /// Soft deletes a post
    async fn delete_post(&self, post_id: i32) -> Result<usize, Self::Error>;

    /// Retrieves a paginated list of posts with user information
    async fn get_posts_with_user(
        &self,
        pagination: &QueryPagination,
    ) -> Result<ListPostResult, Self::Error>;

    /// Retrieves a paginated list of posts for a specific user
    async fn get_posts_by_user(
        &self,
        target_user_id: i32,
        pagination: &QueryPagination,
    ) -> Result<ListPostResult, Self::Error>;

    /// Retrieves a single post with user information
    async fn get_post_with_user(&self, post_id: i32) -> Result<PostPublic, Self::Error>;
}

pub struct PostgresPostRepository {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    type Error = WebError;

    async fn create_post(
        &self,
        owner_user_id: i32,
        post_title: &str,
        post_body: &str,
    ) -> Result<Post, Self::Error> {
        use crate::schema::posts::dsl::*;

        let conn = &mut self.pool.get()?;

        let new_post_data = NewPost {
            title: &post_title,
            body: &post_body,
            published: true,
            user_id: owner_user_id,
        };

        let new_post = diesel::insert_into(crate::schema::posts::table)
            .values(&new_post_data)
            .returning(Post::as_returning())
            .get_result(conn)?;

        Ok(new_post)
    }

    async fn get_post(&self, post_id: i32) -> Result<Post, Self::Error> {
        use crate::schema::posts::dsl::*;

        let conn = &mut self.pool.get()?;

        let post = posts
            .find(post_id)
            .filter(deleted_at.is_null())
            .first(conn)?;

        Ok(post)
    }

    async fn get_posts(&self, pagination: &QueryPagination) -> Result<Vec<Post>, Self::Error> {
        use crate::schema::posts::dsl::*;

        let conn = &mut self.pool.get()?;

        let posts_vec = posts
            .filter(deleted_at.is_null())
            .order(created_at.desc())
            .limit(pagination.limit)
            .offset(pagination.get_offset())
            .load(conn)?;

        Ok(posts_vec)
    }

    async fn update_post(
        &self,
        post_id: i32,
        post_title: &str,
        post_body: &str,
    ) -> Result<Post, Self::Error> {
        use crate::schema::posts::dsl::*;

        let conn = &mut self.pool.get()?;

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

    async fn delete_post(&self, post_id: i32) -> Result<usize, Self::Error> {
        use crate::schema::posts::dsl::*;

        let conn = &mut self.pool.get()?;

        let delete_result = diesel::update(posts.find(post_id))
            .set(deleted_at.eq(diesel::dsl::now))
            .execute(conn)?;

        Ok(delete_result)
    }

    async fn get_posts_with_user(
        &self,
        pagination: &QueryPagination,
    ) -> Result<ListPostResult, Self::Error> {
        use crate::schema::posts::dsl::{created_at, deleted_at, posts};
        use crate::schema::posts::table as post_table;
        use crate::schema::users::dsl::users;

        let conn = &mut self.pool.get()?;

        let posts_raw = posts
            .inner_join(users)
            .filter(deleted_at.is_null())
            .order(created_at.desc())
            .limit(pagination.limit)
            .offset(pagination.get_offset())
            .select((Post::as_select(), User::as_select()))
            .load::<(Post, User)>(conn)?;

        let posts_mapped = posts_raw
            .into_iter()
            .map(|(post, user)| PostPublic {
                user,
                time_human: time_to_human_readable(post.created_at),
                post,
                allow_update: false,
            })
            .collect();

        let total_posts = post_table
            .filter(deleted_at.is_null())
            .count()
            .get_result::<i64>(conn)?;

        Ok(ListPostResult {
            posts: posts_mapped,
            total: total_posts,
        })
    }

    async fn get_posts_by_user(
        &self,
        target_user_id: i32,
        pagination: &QueryPagination,
    ) -> Result<ListPostResult, Self::Error> {
        use crate::schema::posts::dsl::{created_at, deleted_at, posts, user_id};
        use crate::schema::posts::table as post_table;
        use crate::schema::users::dsl::users;

        let conn = &mut self.pool.get()?;

        let posts_raw = posts
            .inner_join(users)
            .filter(user_id.eq(target_user_id))
            .filter(deleted_at.is_null())
            .order(created_at.desc())
            .limit(pagination.limit)
            .offset(pagination.get_offset())
            .select((Post::as_select(), User::as_select()))
            .load::<(Post, User)>(conn)?;

        let posts_mapped = posts_raw
            .into_iter()
            .map(|(post, user)| PostPublic {
                user,
                time_human: time_to_human_readable(post.created_at),
                post,
                allow_update: false,
            })
            .collect();

        let total_posts = post_table
            .filter(user_id.eq(target_user_id))
            .filter(deleted_at.is_null())
            .count()
            .get_result::<i64>(conn)?;

        Ok(ListPostResult {
            posts: posts_mapped,
            total: total_posts,
        })
    }

    async fn get_post_with_user(&self, post_id: i32) -> Result<PostPublic, Self::Error> {
        use crate::schema::posts::dsl::{deleted_at, id, posts};
        use crate::schema::users::table as user_table;

        let conn = &mut self.pool.get()?;

        let (post, user) = posts
            .inner_join(user_table)
            .filter(id.eq(post_id))
            .filter(deleted_at.is_null())
            .first::<(Post, User)>(conn)?;

        let post_public = PostPublic {
            user,
            time_human: time_to_human_readable(post.created_at),
            post,
            allow_update: false,
        };

        Ok(post_public)
    }
}
