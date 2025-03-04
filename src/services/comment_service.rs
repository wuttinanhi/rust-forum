use std::sync::Arc;

use crate::{
    entities::comment::ListCommentResult,
    models::Comment,
    repositories::comment_repository::{CommentRepository, CommentRepositoryWithError},
    utils::pagination::QueryPagination,
};

pub enum CommentServiceError {
    ErrorCreateComment,
    ErrorGetComment,
    ErrorUpdateComment,
    ErrorDeleteComment,
}

pub trait CommentService: Send + Sync {
    /// Creates a new comment
    fn create_comment(
        &self,
        comment_user_id: i32,
        parent_post_id: i32,
        comment_body: &str,
    ) -> Result<Comment, CommentServiceError>;

    /// Retrieves a comment by its ID
    fn get_comment(&self, comment_id: i32) -> Result<Comment, CommentServiceError>;

    /// Retrieves all comments for a post
    fn get_comments(&self, parent_post_id: i32) -> Result<Vec<Comment>, CommentServiceError>;

    /// Updates an existing comment
    fn update_comment(
        &self,
        target_comment_id: i32,
        new_body: &str,
    ) -> Result<Comment, CommentServiceError>;

    /// Soft deletes a comment
    fn delete_comment(&self, target_comment_id: i32) -> Result<usize, CommentServiceError>;

    /// Retrieves comments with user information for a post
    fn get_comments_with_user(
        &self,
        parent_post_id: i32,
        pagination: &QueryPagination,
    ) -> Result<ListCommentResult, CommentServiceError>;

    /// Retrieves comments by a specific user
    fn get_comments_by_user(
        &self,
        target_user_id: i32,
        pagination: &QueryPagination,
    ) -> Result<ListCommentResult, CommentServiceError>;

    /// Gets the page number where a comment appears
    fn get_page_where_comment_at(
        &self,
        target_comment: &Comment,
        page_limit: i64,
    ) -> Result<i64, CommentServiceError>;
}

pub struct BasedCommentService {
    comment_repository: Arc<CommentRepositoryWithError>,
}

impl BasedCommentService {
    pub fn new(comment_repository: Arc<CommentRepositoryWithError>) -> Self {
        Self { comment_repository }
    }
}

impl CommentService for BasedCommentService {
    fn create_comment(
        &self,
        comment_user_id: i32,
        parent_post_id: i32,
        comment_body: &str,
    ) -> Result<Comment, CommentServiceError> {
        todo!()
    }

    fn get_comment(&self, comment_id: i32) -> Result<Comment, CommentServiceError> {
        todo!()
    }

    fn get_comments(&self, parent_post_id: i32) -> Result<Vec<Comment>, CommentServiceError> {
        todo!()
    }

    fn update_comment(
        &self,
        target_comment_id: i32,
        new_body: &str,
    ) -> Result<Comment, CommentServiceError> {
        todo!()
    }

    fn delete_comment(&self, target_comment_id: i32) -> Result<usize, CommentServiceError> {
        todo!()
    }

    fn get_comments_with_user(
        &self,
        parent_post_id: i32,
        pagination: &QueryPagination,
    ) -> Result<ListCommentResult, CommentServiceError> {
        todo!()
    }

    fn get_comments_by_user(
        &self,
        target_user_id: i32,
        pagination: &QueryPagination,
    ) -> Result<ListCommentResult, CommentServiceError> {
        self.comment_repository
            .get_comments_by_user(target_user_id, pagination)
            .map_err(|_| CommentServiceError::ErrorGetComment)
    }

    fn get_page_where_comment_at(
        &self,
        target_comment: &Comment,
        page_limit: i64,
    ) -> Result<i64, CommentServiceError> {
        todo!()
    }
}
