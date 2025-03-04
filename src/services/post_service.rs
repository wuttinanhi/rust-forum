use std::sync::Arc;

use crate::{
    entities::post::{ListPostResult, PostPublic},
    models::Post,
    repositories::post_repository::{PostRepository, PostRepositoryWithError},
    utils::pagination::QueryPagination,
};

pub enum PostServiceError {
    ErrorCreatePost,
    ErrorGetPost,
    ErrorUpdatePost,
    ErrorDeletePost,
}

pub trait PostService: Send + Sync {
    /// Creates a new post
    fn create_post(
        &self,
        owner_user_id: i32,
        post_title: &str,
        post_body: &str,
    ) -> Result<Post, PostServiceError>;

    /// Retrieves a post by its ID
    fn get_post(&self, post_id: i32) -> Result<Post, PostServiceError>;

    /// Retrieves a paginated list of posts
    fn get_posts(&self, pagination: &QueryPagination) -> Result<Vec<Post>, PostServiceError>;

    /// Updates an existing post
    fn update_post(
        &self,
        post_id: i32,
        post_title: &str,
        post_body: &str,
    ) -> Result<Post, PostServiceError>;

    /// Soft deletes a post
    fn delete_post(&self, post_id: i32) -> Result<usize, PostServiceError>;

    /// Retrieves a paginated list of posts with user information
    fn get_posts_with_user(
        &self,
        pagination: &QueryPagination,
    ) -> Result<ListPostResult, PostServiceError>;

    /// Retrieves a paginated list of posts for a specific user
    fn get_posts_by_user(
        &self,
        target_user_id: i32,
        pagination: &QueryPagination,
    ) -> Result<ListPostResult, PostServiceError>;

    /// Retrieves a single post with user information
    fn get_post_with_user(&self, post_id: i32) -> Result<PostPublic, PostServiceError>;
}

pub struct BasedPostService {
    post_repository: Arc<PostRepositoryWithError>,
}

impl BasedPostService {
    pub fn new(post_repository: Arc<PostRepositoryWithError>) -> Self {
        Self { post_repository }
    }
}

impl PostService for BasedPostService {
    fn create_post(
        &self,
        owner_user_id: i32,
        post_title: &str,
        post_body: &str,
    ) -> Result<Post, PostServiceError> {
        todo!()
    }

    fn get_post(&self, post_id: i32) -> Result<Post, PostServiceError> {
        todo!()
    }

    fn get_posts(&self, pagination: &QueryPagination) -> Result<Vec<Post>, PostServiceError> {
        todo!()
    }

    fn update_post(
        &self,
        post_id: i32,
        post_title: &str,
        post_body: &str,
    ) -> Result<Post, PostServiceError> {
        todo!()
    }

    fn delete_post(&self, post_id: i32) -> Result<usize, PostServiceError> {
        todo!()
    }

    fn get_posts_with_user(
        &self,
        pagination: &QueryPagination,
    ) -> Result<ListPostResult, PostServiceError> {
        todo!()
    }

    fn get_posts_by_user(
        &self,
        target_user_id: i32,
        pagination: &QueryPagination,
    ) -> Result<ListPostResult, PostServiceError> {
        self.post_repository
            .get_posts_by_user(target_user_id, pagination)
            .map_err(|_| PostServiceError::ErrorGetPost)
    }

    fn get_post_with_user(&self, post_id: i32) -> Result<PostPublic, PostServiceError> {
        todo!()
    }
}
