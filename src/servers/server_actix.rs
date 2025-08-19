use actix_cors::Cors;
use actix_limitation::{Limiter, RateLimiter};
use actix_multipart::form::MultipartFormConfig;
use actix_session::config::PersistentSession;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::body::{BoxBody, EitherBody};
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::http;
use actix_web::{
    cookie::Key,
    web::{self},
    App,
};
use handlebars::{DirectorySourceOptions, Handlebars};

use crate::controllers::comment_controller::{
    create_comment_submit_route, delete_comment_route, update_comment_post_route,
    update_comment_route,
};
use crate::controllers::post_controller::{
    create_post_route, create_post_submit_route, delete_post_route, index_list_posts_route,
    update_post_route, update_post_submit_route, view_post_route,
};
use crate::controllers::profile_controller::profile_view_route;

use crate::controllers::user_controller::{
    users_changepassword_post_route, users_profile_picture_upload_post_route,
    users_resetpassword_post_route, users_resetpassword_route, users_resetpasswordtoken_post_route,
    users_resetpasswordtoken_route, users_settings_route, users_update_data_post_route,
};
use crate::controllers::user_controller::{
    users_login_post_route, users_login_route, users_logout, users_register_post_route,
    users_register_route,
};

use crate::servers::actix_etc::actix_fallback_error_handler::actix_fallback_error_handler;
use crate::servers::actix_etc::actix_multipart_error_handler::actix_multipart_error_handler;
use crate::utils::pagination::handlebars_pagination_helper;
use crate::AppKit;

use actix_files as fs;
use actix_web::middleware::TrailingSlash;
use actix_web::middleware::{ErrorHandlers, NormalizePath};

type NestedBody = EitherBody<EitherBody<EitherBody<BoxBody, BoxBody>, BoxBody>, BoxBody>;

pub fn create_actix_app(
    app_kit: AppKit,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<NestedBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let app_kit_web_data = web::Data::new(app_kit);

    // Setup CORS
    let cors_origins = std::env::var("APP_CORS_ORIGINS")
        .unwrap_or("http://localhost:3000,http://127.0.0.1:3000".to_string());

    println!("APP_CORS_ORIGINS: {:?}", &cors_origins);

    let cors_origins_split: Vec<String> = cors_origins.split(',').map(|s| s.to_string()).collect();
    let cors_origins_split = cors_origins_split.clone();

    // Setup Rate Limit
    let ratelimit_redis_host = std::env::var("REDIS_HOST").unwrap_or("127.0.0.1".to_string());
    let ratelimit_redis_password = std::env::var("REDIS_PASSWORD").unwrap_or("".to_string());
    let mut ratelimit_redis_url = format!("redis://{ratelimit_redis_host}");

    if !ratelimit_redis_password.is_empty() {
        ratelimit_redis_url = format!("redis://:{ratelimit_redis_password}@{ratelimit_redis_host}");
    }

    println!("ratelimit_redis_url: {:?}", &ratelimit_redis_url);
    // let ratelimit_redis_url = ratelimit_redis_url.clone();

    // -- setup static file directory --
    let static_file_dir_path =
        std::env::var("STATIC_FILE_DIR").unwrap_or("/app/static".to_string());
    // Create static directory if not exists
    std::fs::create_dir_all(&static_file_dir_path).expect("Failed to create static directory");

    // --- handlebars setup ---
    let mut handlebars = Handlebars::new();

    // register handlebars pagination helper
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

    // cookie key must be 64 bytes long
    let cookie_key = Key::from(cookie_key_str.as_bytes());

    let cookie_store = CookieSessionStore::default();

    let cookie_session_middleware = SessionMiddleware::builder(cookie_store, cookie_key)
        .cookie_secure(cookie_secure)
        .session_lifecycle(
            PersistentSession::default().session_ttl(actix_web::cookie::time::Duration::days(7)),
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
        // error handlers
        .wrap(ErrorHandlers::new().default_handler(actix_fallback_error_handler))
        .app_data(MultipartFormConfig::default().error_handler(actix_multipart_error_handler))
        // handlebars
        .app_data(handlebars_ref.clone())
        // limiter
        .wrap(RateLimiter::default())
        .app_data(limiter.clone())
        // path fix
        .wrap(NormalizePath::new(TrailingSlash::Trim))
        // static file serving
        .service(fs::Files::new("/static", &static_file_dir_path))
        // increase payload size
        .app_data(web::PayloadConfig::new(50_000))
        // cors
        .wrap(cors_middleware)
        // cookies
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
}
