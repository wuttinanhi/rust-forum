use std::sync::Arc;

use actix_web::HttpServer;

use rust_forum::db::{MIGRATIONS, establish_connection, initialize_db_pool, run_migrations};
use rust_forum::repositories::comment_repository::PostgresCommentRepository;
use rust_forum::repositories::post_repository::PostgresPostRepository;
use rust_forum::repositories::token_repository::PostgresTokenRepository;
use rust_forum::repositories::user_repository_postgres::PostgresUserRepository;
use rust_forum::servers::server_actix::create_actix_app;
use rust_forum::services::comment_service::BasedCommentService;
use rust_forum::services::email_service::BasedEmailService;
use rust_forum::services::post_service::BasedPostService;
use rust_forum::services::token_service::BasedTokenService;
use rust_forum::services::user_service::BasedUserService;
use rust_forum::{AppKit};

use dotenv::dotenv;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // FOR ENABLE DEBUGGING
    // std::env::set_var("RUST_LOG", "debug");

    env_logger::init();

    // get mode from env
    // let app_mode = std::env::var("ENV").unwrap_or("dev".to_string());

    let host = std::env::var("APP_HOST").unwrap_or("0.0.0.0".to_string());

    let port = std::env::var("APP_PORT")
        .unwrap_or("3000".to_string())
        .parse()
        .unwrap_or(3000);

    println!("listening at http://{}:{}", host, port);

    let worker_count = std::env::var("APP_WORKER_COUNT")
        .unwrap_or("10".to_string())
        .parse()
        .expect("failed to parse APP_WORKER_COUNT");

    println!("APP_WORKER_COUNT={}", worker_count);

    // --- Create static directory if not exists ---
    std::fs::create_dir_all("./static").expect("Failed to create static directory");

    // --- database setup ---
    let db_pool = initialize_db_pool();

    let db_pool_arc = Arc::new(db_pool.clone());

    // run migrations
    let mut conn = establish_connection();
    run_migrations(&mut conn, MIGRATIONS).expect("failed to run migrations");

    // --- repository setup ---
    //
    // Note:
    //
    // this is not equal
    // let comment_repo_web_data: Data<Arc<dyn CommentRepository<Error = Box<dyn Error + Send + Sync>>>>
    // --- and ---
    // let comment_repo_web_data: Data<Arc<PostgresCommentRepository>>
    //
    // the web::Data in route will unable to extract if no type annotate specified
    //
    // let post_repo_web_data: actix_web::web::Data<Arc<dyn PostRepository<Error = WebError>>> =
    //     actix_web::web::Data::new(Arc::new(post_repo));
    //
    // optional after Arc::new
    // as Arc<dyn PostRepository<Error = WebError>>

    let post_repo = PostgresPostRepository::new(db_pool_arc.clone());
    let post_repo = Arc::new(post_repo);

    let comment_repo = PostgresCommentRepository::new(db_pool_arc.clone());
    // let comment_repo_web_data: actix_web::web::Data<Arc<dyn CommentRepository<Error = WebError>>> =
    //     actix_web::web::Data::new(Arc::new(comment_repo));
    let comment_repo = Arc::new(comment_repo);

    let token_repo = PostgresTokenRepository::new(db_pool_arc.clone());
    let token_repo = Arc::new(token_repo);

    let user_repo = PostgresUserRepository::new(db_pool_arc.clone());
    let user_repo = Arc::new(user_repo);

    // --- service setup ---
    let token_service = BasedTokenService::new(token_repo.clone());
    let token_service = Arc::new(token_service);

    let email_service = BasedEmailService::new();
    let email_service = Arc::new(email_service);

    let user_service = BasedUserService::new(user_repo.clone(), token_repo.clone());
    let user_service = Arc::new(user_service);

    let post_service = BasedPostService::new(post_repo.clone());
    let post_service = Arc::new(post_service);

    let comment_service = BasedCommentService::new(comment_repo.clone());
    let comment_service = Arc::new(comment_service);

    // Setup CORS
    let cors_origins_env = std::env::var("APP_CORS_ORIGINS")
        .unwrap_or("http://localhost:3000,http://127.0.0.1:3000".to_string());

    println!("APP_CORS_ORIGINS: {:?}", &cors_origins_env);

    let cors_origins_vec: Vec<String> =
        cors_origins_env.split(',').map(|s| s.to_string()).collect();

    // Setup Redis Rate Limit
    let ratelimit_redis_host = std::env::var("REDIS_HOST").unwrap_or("127.0.0.1".to_string());
    let ratelimit_redis_password = std::env::var("REDIS_PASSWORD").unwrap_or("".to_string());
    let redis_ratelimit_url = if ratelimit_redis_password.is_empty() {
        format!("redis://{ratelimit_redis_host}")
    } else {
        format!("redis://default:{ratelimit_redis_password}@{ratelimit_redis_host}")
    };

    println!("ratelimit_redis_url: {:?}", &redis_ratelimit_url);

    // -- setup static file directory --
    let static_file_dir_path = std::env::var("STATIC_FILE_DIR").unwrap_or("".to_string());
    if static_file_dir_path.is_empty() {
        panic!("env STATIC_FILE_DIR is not set!");
    }
    // Create static directory if not exists
    std::fs::create_dir_all(&static_file_dir_path).expect("Failed to create static directory");
    println!("create STATIC_FILE_DIR at {:?}", &static_file_dir_path);

    // --- app kit setup ---
    let app_kit = AppKit {
        user_service: user_service.clone(),
        email_service: email_service.clone(),
        token_service: token_service.clone(),
        post_service: post_service.clone(),
        comment_service: comment_service.clone(),
        cors_origins: cors_origins_vec,
        redis_ratelimit_url,
        static_file_dir_path,
    };

    HttpServer::new(move || {
        let app_kit_clone = app_kit.clone();
        create_actix_app(app_kit_clone)
    })
    .bind((host, port))?
    .workers(worker_count)
    .run()
    .await
}
