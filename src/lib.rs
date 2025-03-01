pub mod models;
pub mod schema;

pub mod routes;

pub mod comments;
pub mod controllers;
pub mod db;
pub mod entities;
pub mod posts;
pub mod repositories;
pub mod services;
pub mod users;
pub mod utils;

use diesel::{Connection, PgConnection};
use repositories::{
    comment_repository::CommentRepositoryWithError, post_repository::PostRepositoryWithError,
    user_repository::UserRepositoryWithError,
};
use services::{
    email_service::EmailService, token_service::TokenService, user_service::UserService,
};
use std::{env, sync::Arc};

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[derive(Clone)]
pub struct AppKit {
    pub post_repository: Arc<PostRepositoryWithError>,
    pub comment_repository: Arc<CommentRepositoryWithError>,
    pub user_repository: Arc<UserRepositoryWithError>,

    pub token_service: Arc<dyn TokenService>,
    pub email_service: Arc<dyn EmailService>,

    pub user_service: Arc<dyn UserService>,
}
