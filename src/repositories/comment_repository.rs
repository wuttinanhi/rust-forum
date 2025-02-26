use crate::db::WebError;
use crate::entities::comment::{CommentPublic, ListCommentResult};
use crate::models::{Comment, NewComment, User};
use crate::utils::pagination::QueryPagination;
use crate::utils::time::time_to_human_readable;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;

pub trait CommentRepository: Send + Sync {
    type Error;

    /// Creates a new comment
    fn create_comment(
        &self,
        comment_user_id: i32,
        parent_post_id: i32,
        comment_body: &str,
    ) -> Result<Comment, Self::Error>;

    /// Retrieves a comment by its ID
    fn get_comment(&self, comment_id: i32) -> Result<Comment, Self::Error>;

    /// Retrieves all comments for a post
    fn get_comments(&self, parent_post_id: i32) -> Result<Vec<Comment>, Self::Error>;

    /// Updates an existing comment
    fn update_comment(
        &self,
        target_comment_id: i32,
        new_body: &str,
    ) -> Result<Comment, Self::Error>;

    /// Soft deletes a comment
    fn delete_comment(&self, target_comment_id: i32) -> Result<usize, Self::Error>;

    /// Retrieves comments with user information for a post
    fn get_comments_with_user(
        &self,
        parent_post_id: i32,
        pagination: &QueryPagination,
    ) -> Result<ListCommentResult, Self::Error>;

    /// Retrieves comments by a specific user
    fn get_comments_by_user(
        &self,
        target_user_id: i32,
        pagination: &QueryPagination,
    ) -> Result<ListCommentResult, Self::Error>;

    /// Gets the page number where a comment appears
    fn get_page_where_comment_at(
        &self,
        target_comment: &Comment,
        page_limit: i64,
    ) -> Result<i64, Self::Error>;
}

pub struct PostgresCommentRepository {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

impl PostgresCommentRepository {
    pub fn new(pool: Arc<Pool<ConnectionManager<PgConnection>>>) -> Self {
        Self { pool }
    }
}

impl CommentRepository for PostgresCommentRepository {
    type Error = WebError;

    fn create_comment(
        &self,
        comment_user_id: i32,
        parent_post_id: i32,
        comment_body: &str,
    ) -> Result<Comment, Self::Error> {
        use crate::schema::comments::table as comments_table;

        let mut conn = self.pool.get()?;

        let new_comment_data = NewComment {
            post_id: parent_post_id,
            user_id: comment_user_id,
            content: comment_body,
        };

        let new_comment = diesel::insert_into(comments_table)
            .values(&new_comment_data)
            .returning(Comment::as_returning())
            .get_result(&mut conn)?;

        Ok(new_comment)
    }

    fn get_comment(&self, comment_id: i32) -> Result<Comment, Self::Error> {
        let mut conn = self.pool.get()?;
        use crate::schema::comments::dsl::*;

        let comment = comments
            .find(comment_id)
            .filter(deleted_at.is_null())
            .first(&mut conn)?;

        Ok(comment)
    }

    fn get_comments(&self, parent_post_id: i32) -> Result<Vec<Comment>, Self::Error> {
        let mut conn = self.pool.get()?;

        use crate::schema::comments::dsl::*;

        let comments_vec = comments
            .filter(post_id.eq(parent_post_id))
            .filter(deleted_at.is_null())
            .order(created_at.desc())
            .load(&mut conn)?;

        Ok(comments_vec)
    }

    fn update_comment(
        &self,
        target_comment_id: i32,
        new_body: &str,
    ) -> Result<Comment, Self::Error> {
        let mut conn = self.pool.get()?;

        use crate::schema::comments::dsl::*;

        let comment = diesel::update(comments.find(target_comment_id))
            .set(content.eq(new_body))
            .returning(Comment::as_returning())
            .get_result(&mut conn)?;

        Ok(comment)
    }

    fn delete_comment(&self, target_comment_id: i32) -> Result<usize, Self::Error> {
        let mut conn = self.pool.get()?;

        use crate::schema::comments::dsl::*;

        let delete_usize = diesel::update(comments.find(target_comment_id))
            .set(deleted_at.eq(diesel::dsl::now))
            .execute(&mut conn)?;

        Ok(delete_usize)
    }

    fn get_comments_with_user(
        &self,
        parent_post_id: i32,
        pagination: &QueryPagination,
    ) -> Result<ListCommentResult, Self::Error> {
        let mut conn = self.pool.get()?;

        use crate::schema::comments::dsl::{comments, created_at, deleted_at, post_id};
        use crate::schema::comments::table as comments_table;
        use crate::schema::users::dsl::users;

        let comments_joined = comments
            .inner_join(users)
            .filter(post_id.eq(parent_post_id))
            .filter(deleted_at.is_null())
            .order(created_at.asc())
            .limit(pagination.limit)
            .offset(pagination.get_offset())
            .select((Comment::as_select(), User::as_select()))
            .load::<(Comment, User)>(&mut conn)?;

        let comments_mapped = comments_joined
            .into_iter()
            .map(|(comment, user)| CommentPublic {
                time_human: time_to_human_readable(comment.created_at),
                comment,
                user,
                allow_update: false,
            })
            .collect();

        let total = comments_table
            .filter(post_id.eq(parent_post_id))
            .filter(deleted_at.is_null())
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(ListCommentResult {
            comments: comments_mapped,
            total,
        })
    }

    fn get_comments_by_user(
        &self,
        target_user_id: i32,
        pagination: &QueryPagination,
    ) -> Result<ListCommentResult, Self::Error> {
        let mut conn = self.pool.get()?;

        use crate::schema::comments::dsl::{comments, created_at, deleted_at, user_id};
        use crate::schema::comments::table as comments_table;
        use crate::schema::users::dsl::users;

        let comments_joined = comments
            .inner_join(users)
            .filter(user_id.eq(target_user_id))
            .filter(deleted_at.is_null())
            .order(created_at.desc())
            .limit(pagination.limit)
            .offset(pagination.get_offset())
            .select((Comment::as_select(), User::as_select()))
            .load::<(Comment, User)>(&mut conn)?;

        let comments_mapped = comments_joined
            .into_iter()
            .map(|(comment, user)| CommentPublic {
                time_human: time_to_human_readable(comment.created_at),
                comment,
                user,
                allow_update: false,
            })
            .collect();

        let total = comments_table
            .filter(user_id.eq(target_user_id))
            .filter(deleted_at.is_null())
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(ListCommentResult {
            comments: comments_mapped,
            total,
        })
    }

    fn get_page_where_comment_at(
        &self,
        target_comment: &Comment,
        page_limit: i64,
    ) -> Result<i64, Self::Error> {
        let mut conn = self.pool.get()?;

        use crate::schema::comments::dsl::{deleted_at, id, post_id};
        use crate::schema::comments::table as comments_table;

        let nth_row_comment = comments_table
            .filter(post_id.eq(target_comment.post_id))
            .filter(deleted_at.is_null())
            .order(id.asc())
            .select(id)
            .load::<i32>(&mut conn)?
            .iter()
            .position(|&comment_id_value| comment_id_value == target_comment.id)
            .map(|pos| pos as i64 + 1)
            .unwrap_or(0);

        let page = (nth_row_comment as f64 / page_limit as f64).ceil() as i64;

        Ok(page)
    }
}
