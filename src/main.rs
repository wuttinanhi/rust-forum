use actix_cors::Cors;
use actix_limitation::{Limiter, RateLimiter};
use actix_session::config::PersistentSession;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::dev::ServiceRequest;
use actix_web::http;
use actix_web::middleware::ErrorHandlers;
use actix_web::{
    cookie::Key,
    web::{self},
    App, HttpServer,
};
use handlebars::{DirectorySourceOptions, Handlebars};
use rust_forum::routes::error::fallback_error_handler;
use rust_forum::users::route::{
    users_changepassword_post_route, users_settings_route, users_update_data_post_route,
};
use rust_forum::{
    comments::routes::create_comment_submit_route,
    db::initialize_db_pool,
    posts::route::{
        create_post_route, create_post_submit_route, index_list_posts_route, view_post_route,
    },
    users::route::{
        users_login_post_route, users_login_route, users_logout, users_register_post_route,
        users_register_route,
    },
};

use actix_web::middleware::NormalizePath;
use actix_web::middleware::TrailingSlash;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
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

    // CORS
    let cors_origins = std::env::var("APP_CORS_ORIGINS")
        .unwrap_or("http://localhost:3000,http://127.0.0.1:3000".to_string());

    println!("APP_CORS_ORIGINS: {:?}", &cors_origins);

    let cors_origins_split: Vec<String> = cors_origins.split(',').map(|s| s.to_string()).collect();

    // RATE LIMIT
    let ratelimit_redis_host = std::env::var("REDIS_HOST").unwrap_or("127.0.0.1".to_string());
    let ratelimit_redis_password = std::env::var("REDIS_PASSWORD").unwrap_or("".to_string());
    let mut ratelimit_redis_url = format!("redis://{ratelimit_redis_host}");

    if !ratelimit_redis_password.is_empty() {
        ratelimit_redis_url = format!("redis://:{ratelimit_redis_password}@{ratelimit_redis_host}");
    }

    println!("ratelimit_redis_url: {:?}", &ratelimit_redis_url);

    HttpServer::new(move || {
        let ratelimit_redis_url = ratelimit_redis_url.clone();

        let cors_origins_split = cors_origins_split.clone();

        // let db = initialize_db_pool();
        let db_ref = web::Data::new(db_pool.clone());

        // --- handlebars setup ---
        let mut handlebars = Handlebars::new();

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
        let cookie_secure = std::env::var("COOKIE_SECURE")
            .map(|val| val == "true")
            .unwrap_or(false);

        // --- cookie session middleware ---

        let cookie_key = Key::from(
            // must be 64 bytes long
            "dev-cookie-key-1234567890-dev-cookie-key-1234567890-dev-cookie-key-1234567890"
                .as_bytes(),
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
            .service(users_settings_route);

        let posts_scope = web::scope("/posts")
            .service(create_post_route)
            .service(create_post_submit_route)
            .service(view_post_route)
            .route("", web::to(index_list_posts_route));

        let comments_scope = web::scope("/comments").service(create_comment_submit_route);

        // --- init app ---
        App::new()
            .app_data(db_ref.clone())
            .app_data(handlebars_ref.clone())
            .wrap(RateLimiter::default())
            .app_data(limiter.clone())
            .wrap(ErrorHandlers::new().default_handler(fallback_error_handler))
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(cors_middleware)
            .wrap(cookie_session_middleware)
            .service(users_scope)
            .service(posts_scope)
            .service(comments_scope)
            .route("/", web::to(index_list_posts_route))
    })
    .bind((host, port))?
    // .workers(1)
    .run()
    .await
}
