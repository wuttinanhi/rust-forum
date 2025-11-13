pub mod models;
pub mod schema;

pub mod db;

pub mod controllers;
pub mod entities;
pub mod repositories;
pub mod services;

pub mod utils;

pub mod servers;

mod errors;
pub mod handlebars_helper;
pub mod macros;
pub mod tests;

use db::initialize_db_pool;
use diesel::{Connection, PgConnection};

use repositories::{
    comment_repository::PostgresCommentRepository, post_repository::PostgresPostRepository,
    token_repository::PostgresTokenRepository, user_repository_inmemory::InMemoryUserRepository,
};
use services::{
    comment_service::{BasedCommentService, CommentService},
    email_service::{BasedEmailService, EmailService},
    post_service::{BasedPostService, PostService},
    token_service::{BasedTokenService, TokenService},
    user_service::{BasedUserService, UserService},
};
use std::sync::Arc;

pub fn establish_connection() -> PgConnection {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[derive(Clone)]
pub struct AppKit {
    // pub post_repository: Arc<PostRepositoryWithError>,
    // pub comment_repository: Arc<CommentRepositoryWithError>,
    // pub user_repository: Arc<UserRepositoryWithError>,
    pub token_service: Arc<dyn TokenService>,
    pub email_service: Arc<dyn EmailService>,

    pub user_service: Arc<dyn UserService>,
    pub post_service: Arc<dyn PostService>,
    pub comment_service: Arc<dyn CommentService>,

    pub cors_origins: Vec<String>,
    pub redis_ratelimit_url: String,
    pub static_file_dir_path: String,
}

impl AppKit {
    pub fn new_for_testing() -> Self {
        // clear turnstile settings
        std::env::remove_var("CLOUDFLARE_TURNSTILE_SECRET_KEY");
        std::env::remove_var("CLOUDFLARE_TURNSTILE_SITE_KEY");

        // --- database setup ---
        let db_pool = initialize_db_pool();

        let db_pool_arc = Arc::new(db_pool.clone());

        let post_repo = PostgresPostRepository::new(db_pool_arc.clone());
        let post_repo_arc = Arc::new(post_repo);

        let comment_repo = PostgresCommentRepository::new(db_pool_arc.clone());
        // let comment_repo_web_data: actix_web::web::Data<Arc<dyn CommentRepository<Error = WebError>>> =
        //     actix_web::web::Data::new(Arc::new(comment_repo));
        let comment_repo_arc = Arc::new(comment_repo);

        let token_repo = PostgresTokenRepository::new(db_pool_arc.clone());
        let token_repo_arc = Arc::new(token_repo);

        // let user_repo = PostgresUserRepository::new(db_pool_arc.clone());

        let user_repo_inmemory = InMemoryUserRepository::new();
        let user_repo_arc = Arc::new(user_repo_inmemory);

        // --- service setup ---
        let token_service = BasedTokenService::new(token_repo_arc.clone());
        let email_service = BasedEmailService::new();
        let user_service = BasedUserService::new(user_repo_arc.clone(), token_repo_arc.clone());
        let post_service = BasedPostService::new(post_repo_arc.clone());
        let comment_service = BasedCommentService::new(comment_repo_arc.clone());

        // --- app kit setup ---

        AppKit {
            user_service: Arc::new(user_service),
            email_service: Arc::new(email_service),
            token_service: Arc::new(token_service),
            post_service: Arc::new(post_service),
            comment_service: Arc::new(comment_service),
            cors_origins: vec![
                "http://localhost:3000".to_string(),
                "http://127.0.0.1:3000".to_string(),
            ],
            redis_ratelimit_url: "redis://default:redis123@127.0.0.1".to_string(),
            static_file_dir_path: "./static".to_string(),
        }
    }

    pub fn new() -> Self {
        // --- database setup ---
        let db_pool = initialize_db_pool();

        let db_pool_arc = Arc::new(db_pool.clone());

        let post_repo = PostgresPostRepository::new(db_pool_arc.clone());
        let post_repo_arc = Arc::new(post_repo);

        let comment_repo = PostgresCommentRepository::new(db_pool_arc.clone());
        // let comment_repo_web_data: actix_web::web::Data<Arc<dyn CommentRepository<Error = WebError>>> =
        //     actix_web::web::Data::new(Arc::new(comment_repo));
        let comment_repo_arc = Arc::new(comment_repo);

        let token_repo = PostgresTokenRepository::new(db_pool_arc.clone());
        let token_repo_arc = Arc::new(token_repo);

        // let user_repo = PostgresUserRepository::new(db_pool_arc.clone());

        let user_repo_in_memory = InMemoryUserRepository::new();
        let user_repo_arc = Arc::new(user_repo_in_memory);

        // --- service setup ---
        let token_service = BasedTokenService::new(token_repo_arc.clone());
        let email_service = BasedEmailService::new();
        let user_service = BasedUserService::new(user_repo_arc.clone(), token_repo_arc.clone());
        let post_service = BasedPostService::new(post_repo_arc.clone());
        let comment_service = BasedCommentService::new(comment_repo_arc.clone());

        // --- app kit setup ---

        AppKit {
            user_service: Arc::new(user_service),
            email_service: Arc::new(email_service),
            token_service: Arc::new(token_service),
            post_service: Arc::new(post_service),
            comment_service: Arc::new(comment_service),
            cors_origins: vec![],
            redis_ratelimit_url: "".to_string(),
            static_file_dir_path: "./static".to_string(),
        }
    }
}
