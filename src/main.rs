use std::sync::Arc;

use actix_cors::Cors;
use actix_limitation::{Limiter, RateLimiter};
use actix_multipart::form::MultipartFormConfig;
use actix_session::config::PersistentSession;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::dev::ServiceRequest;
use actix_web::{
    cookie::Key,
    web::{self},
    App, HttpServer,
};
use actix_web::{http, HttpRequest, HttpResponse};
use handlebars::{DirectorySourceOptions, Handlebars};

use rust_forum::controllers::comment_controller::{
    create_comment_submit_route, delete_comment_route, update_comment_post_route,
    update_comment_route,
};
use rust_forum::controllers::post_controller::{
    create_post_route, create_post_submit_route, delete_post_route, index_list_posts_route,
    update_post_route, update_post_submit_route, view_post_route,
};
use rust_forum::controllers::profile_controller::profile_view_route;
use rust_forum::db::run_migrations;

use rust_forum::repositories::comment_repository::PostgresCommentRepository;
use rust_forum::repositories::post_repository::PostgresPostRepository;
use rust_forum::repositories::token_repository::PostgresTokenRepository;
use rust_forum::repositories::user_repository::PostgresUserRepository;
use rust_forum::routes::error_handler::error_handler;
use rust_forum::services::comment_service::BasedCommentService;
use rust_forum::services::email_service::BasedEmailService;
use rust_forum::services::post_service::BasedPostService;
use rust_forum::services::token_service::BasedTokenService;
use rust_forum::services::user_service::BasedUserService;
use rust_forum::users::route::{
    users_changepassword_post_route, users_profile_picture_upload_post_route,
    users_resetpassword_post_route, users_resetpassword_route, users_resetpasswordtoken_post_route,
    users_resetpasswordtoken_route, users_settings_route, users_update_data_post_route,
};
use rust_forum::utils::pagination::handlebars_pagination_helper;
use rust_forum::AppKit;
use rust_forum::{
    db::initialize_db_pool,
    users::route::{
        users_login_post_route, users_login_route, users_logout, users_register_post_route,
        users_register_route,
    },
};

use actix_files as fs;
use actix_web::middleware::TrailingSlash;
use actix_web::middleware::{ErrorHandlers, NormalizePath};
use dotenv::dotenv;

use diesel_migrations::{embed_migrations, EmbeddedMigrations};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/");

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

    // --- database setup ---
    let db_pool = initialize_db_pool();

    let db_pool_arc = Arc::new(db_pool.clone());

    // --- repository setup ---
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
    let post_repo_arc = Arc::new(post_repo);

    let comment_repo = PostgresCommentRepository::new(db_pool_arc.clone());
    // let comment_repo_web_data: actix_web::web::Data<Arc<dyn CommentRepository<Error = WebError>>> =
    //     actix_web::web::Data::new(Arc::new(comment_repo));
    let comment_repo_arc = Arc::new(comment_repo);

    let token_repo = PostgresTokenRepository::new(db_pool_arc.clone());
    let token_repo_arc = Arc::new(token_repo);

    let user_repo = PostgresUserRepository::new(db_pool_arc.clone());
    let user_repo_arc = Arc::new(user_repo);

    // --- service setup ---
    let token_service = BasedTokenService::new(token_repo_arc.clone());
    let email_service = BasedEmailService::new();
    let user_service = BasedUserService::new(user_repo_arc.clone(), token_repo_arc.clone());
    let post_service = BasedPostService::new(post_repo_arc.clone());
    let comment_service = BasedCommentService::new(comment_repo_arc.clone());

    // --- app kit setup ---
    let app_kit = AppKit {
        // post_repository: post_repo_arc.clone(),
        // comment_repository: comment_repo_arc.clone(),
        // user_repository: user_repo_arc.clone(),
        user_service: Arc::new(user_service),
        email_service: Arc::new(email_service),
        token_service: Arc::new(token_service),
        post_service: Arc::new(post_service),
        comment_service: Arc::new(comment_service),
    };

    let app_kit_web_data = web::Data::new(app_kit);

    // Start run auto migrations
    let migration_conn = db_pool_arc.clone();
    {
        let mut conn = migration_conn
            .get()
            .expect("faild to get migration connection");
        let _ = run_migrations(&mut conn, MIGRATIONS);
    }

    // Setup CORS
    let cors_origins = std::env::var("APP_CORS_ORIGINS")
        .unwrap_or("http://localhost:3000,http://127.0.0.1:3000".to_string());

    println!("APP_CORS_ORIGINS: {:?}", &cors_origins);

    let cors_origins_split: Vec<String> = cors_origins.split(',').map(|s| s.to_string()).collect();

    // Setup Rate Limit
    let ratelimit_redis_host = std::env::var("REDIS_HOST").unwrap_or("127.0.0.1".to_string());
    let ratelimit_redis_password = std::env::var("REDIS_PASSWORD").unwrap_or("".to_string());
    let mut ratelimit_redis_url = format!("redis://{ratelimit_redis_host}");

    if !ratelimit_redis_password.is_empty() {
        ratelimit_redis_url = format!("redis://:{ratelimit_redis_password}@{ratelimit_redis_host}");
    }

    println!("ratelimit_redis_url: {:?}", &ratelimit_redis_url);

    // Create static directory if not exists
    std::fs::create_dir_all("./static").expect("Failed to create static directory");

    HttpServer::new(move || {
        let ratelimit_redis_url = ratelimit_redis_url.clone();

        let cors_origins_split = cors_origins_split.clone();

        // init db ref
        let db_ref = web::Data::new(db_pool.clone());

        // --- handlebars setup ---
        let mut handlebars = Handlebars::new();

        handlebars.register_helper("pagination", Box::new(handlebars_pagination_helper));

        // set handlebars options
        let mut handlebars_options = DirectorySourceOptions::default();
        handlebars_options.tpl_extension = ".hbs".to_owned();
        handlebars_options.hidden = false;
        handlebars_options.temporary = false;

        // set handlebars templates directory
        handlebars
            .register_templates_directory("./templates", handlebars_options)
            .unwrap();

        let handlebars_ref = web::Data::new(handlebars);

        // --- cookie session middleware ---
        let cookie_secure = true;
        // std::env::var("COOKIE_SECURE")
        //     .map(|val| val == "true")
        //     .unwrap_or(false);

        let cookie_key_str = std::env::var("COOKIE_KEY").unwrap_or_else(|_| {
            if cfg!(debug_assertions) {
                // Fallback for development
                "dev-cookie-key-1234567890-dev-cookie-key-1234567890-dev-cookie-key-1234567890"
                    .to_string()
            } else {
                // released build
                panic!("COOKIE_KEY environment variable must be set in production!")
            }
        });

        let cookie_key = Key::from(
            // must be 64 bytes long
            cookie_key_str.as_bytes(),
        );

        let cookie_store = CookieSessionStore::default();

        let cookie_session_middleware = SessionMiddleware::builder(cookie_store, cookie_key)
            .cookie_secure(cookie_secure)
            .session_lifecycle(
                PersistentSession::default()
                    .session_ttl(actix_web::cookie::time::Duration::days(7)),
            )
            .build();

        // -- setup CORS ---
        let cors_middleware = Cors::default()
            .allowed_origin_fn(move |origin, _req_head| {
                cors_origins_split
                    .iter()
                    .any(|allowed_origin| allowed_origin == origin)
            })
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                http::header::CONTENT_TYPE,
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
            ])
            .max_age(3600);

        // -- setup rate limiter --
        let limiter = web::Data::new(
            Limiter::builder(ratelimit_redis_url)
                .key_by(|req: &ServiceRequest| {
                    let conn_info = req.connection_info();

                    let forwarded_for = conn_info.realip_remote_addr();
                    if let Some(ip) = forwarded_for {
                        return Some(ip.to_string());
                    }

                    // let cookie_value = req
                    //     .get_session()
                    //     .get(&"session-id")
                    //     .unwrap_or_else(|_| req.cookie(&"rate-api-id").map(|c| c.to_string()));

                    req.peer_addr().map(|addr| addr.ip().to_string())
                })
                .limit(5000)
                .period(std::time::Duration::from_secs(3600)) // 60 minutes
                .build()
                .unwrap(),
        );

        // --- setup routes ---
        let users_scope = web::scope("/users")
            .service(users_login_route)
            .service(users_login_post_route)
            .service(users_register_route)
            .service(users_register_post_route)
            .service(users_logout)
            .service(users_changepassword_post_route)
            .service(users_update_data_post_route)
            .service(users_profile_picture_upload_post_route)
            .service(users_settings_route)
            .service(users_resetpassword_route)
            .service(users_resetpassword_post_route)
            .service(users_resetpasswordtoken_route)
            .service(users_resetpasswordtoken_post_route);

        let posts_scope = web::scope("/posts")
            .service(create_post_route)
            .service(create_post_submit_route)
            .service(view_post_route)
            .service(update_post_route)
            .service(update_post_submit_route)
            .service(delete_post_route)
            .route("", web::to(index_list_posts_route));

        let comments_scope = web::scope("/comments")
            .service(create_comment_submit_route)
            .service(update_comment_route)
            .service(update_comment_post_route)
            .service(delete_comment_route);

        let profile_scope = web::scope("/profile")
            .route("/{user_id}", web::get().to(profile_view_route))
            .route(
                "/{user_id}/{fetch_mode:.*}",
                web::get().to(profile_view_route),
            );

        // --- init app ---
        App::new()
            .wrap(ErrorHandlers::new().default_handler(error_handler))
            .app_data(db_ref.clone())
            .app_data(handlebars_ref.clone())
            .wrap(RateLimiter::default())
            .app_data(limiter.clone())
            .app_data(MultipartFormConfig::default().error_handler(handle_multipart_error))
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .service(fs::Files::new("/static", "./static"))
            // increase payload size
            .app_data(web::PayloadConfig::new(50_000))
            .wrap(cors_middleware)
            .wrap(cookie_session_middleware)
            // app kit
            .app_data(app_kit_web_data.clone())
            // --- route ---
            .service(users_scope)
            .service(posts_scope)
            .service(comments_scope)
            .service(profile_scope)
            // default to posts view route
            .route("/", web::to(index_list_posts_route))
    })
    .bind((host, port))?
    // .workers(1)
    .run()
    .await
}

// apply large file upload fix from https://github.com/actix/actix-web/issues/3152#issuecomment-2539018905
fn handle_multipart_error(
    err: actix_multipart::MultipartError,
    _req: &HttpRequest,
) -> actix_web::Error {
    let response = HttpResponse::BadRequest().force_close().finish();
    actix_web::error::InternalError::from_response(err, response).into()
}
