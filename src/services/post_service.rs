use std::sync::Arc;

use crate::{
    entities::post::{ListPostResult, PostPublic},
    models::Post,
    repositories::post_repository::PostRepositoryWithError,
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
        self.post_repository
            .create_post(owner_user_id, post_title, post_body)
            .map_err(|_| PostServiceError::ErrorCreatePost)
    }

    fn get_post(&self, post_id: i32) -> Result<Post, PostServiceError> {
        self.post_repository
            .get_post(post_id)
            .map_err(|_| PostServiceError::ErrorGetPost)
    }

    fn get_posts(&self, pagination: &QueryPagination) -> Result<Vec<Post>, PostServiceError> {
        self.post_repository
            .get_posts(pagination)
            .map_err(|_| PostServiceError::ErrorGetPost)
    }

    fn update_post(
        &self,
        post_id: i32,
        post_title: &str,
        post_body: &str,
    ) -> Result<Post, PostServiceError> {
        self.post_repository
            .update_post(post_id, post_title, post_body)
            .map_err(|_| PostServiceError::ErrorUpdatePost)
    }

    fn delete_post(&self, post_id: i32) -> Result<usize, PostServiceError> {
        self.post_repository
            .delete_post(post_id)
            .map_err(|_| PostServiceError::ErrorDeletePost)
    }

    fn get_posts_with_user(
        &self,
        pagination: &QueryPagination,
    ) -> Result<ListPostResult, PostServiceError> {
        self.post_repository
            .get_posts_with_user(pagination)
            .map_err(|_| PostServiceError::ErrorGetPost)
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
        self.post_repository
            .get_post_with_user(post_id)
            .map_err(|_| PostServiceError::ErrorGetPost)
    }
}
