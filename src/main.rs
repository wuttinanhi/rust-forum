use actix_session::config::PersistentSession;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::time::Duration;
// use actix_web::middleware::NormalizePath;
// use actix_web::middleware::TrailingSlash;
use actix_web::{
    cookie::Key,
    web::{self},
    App, HttpServer,
};
use handlebars::{DirectorySourceOptions, Handlebars};
use rust_forum::{
    comments::routes::create_comment_submit_route,
    establish_connection,
    posts::route::{
        create_post_route, create_post_submit_route, index_list_posts_route, view_post_route,
    },
    users::route::{
        users_login, users_login_post, users_logout, users_register, users_register_post,
    },
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let host = std::env::var("APP_HOST").unwrap_or("0.0.0.0".to_string());

    let port = std::env::var("APP_PORT")
        .unwrap_or("3000".to_string())
        .parse()
        .unwrap_or(3000);

    println!("listening at {}:{}", host, port);

    // get mode from env
    // let app_mode = std::env::var("ENV").unwrap_or("dev".to_string());
    // let db = initialize_db_pool();

    let mut handlebars_options = DirectorySourceOptions::default();
    handlebars_options.tpl_extension = ".hbs".to_owned();
    handlebars_options.hidden = false;
    handlebars_options.temporary = false;

    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory("templates", handlebars_options)
        .unwrap();
    let handlebars_ref = web::Data::new(handlebars);

    // let pool = config.pg.create_pool(None, NoTls).unwrap();

    HttpServer::new(move || {
        let db = establish_connection();
        let db_ref = web::Data::new(db);

        // --- cookie session middleware ---
        let cookie_key = Key::from(
            // must be 64 bytes long
            "dev-cookie-key-1234567890-dev-cookie-key-1234567890-dev-cookie-key-1234567890"
                .as_bytes(),
        );

        let cookie_secure = std::env::var("COOKIE_SECURE")
            .map(|val| val == "true")
            .unwrap_or(false);

        let cookie_store = CookieSessionStore::default();

        let cookie_session_middleware = SessionMiddleware::builder(cookie_store, cookie_key)
            .cookie_secure(cookie_secure)
            .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(7)))
            .build();

        // --- routes ---
        let users_scope = web::scope("/users")
            .service(users_login)
            .service(users_login_post)
            .service(users_register)
            .service(users_register_post)
            .service(users_logout);

        let posts_scope = web::scope("/posts")
            .service(create_post_route)
            .service(create_post_submit_route)
            .service(view_post_route)
            .service(index_list_posts_route);

        let comments_scope = web::scope("/comments").service(create_comment_submit_route);

        // --- init app ---
        App::new()
            .app_data(db_ref.clone())
            .app_data(handlebars_ref.clone())
            // .wrap(NormalizePath::new(TrailingSlash::Always))
            .wrap(cookie_session_middleware)
            .service(users_scope)
            .service(posts_scope)
            .service(comments_scope)
            .service(index_list_posts_route)
    })
    .bind((host, port))?
    .run()
    .await
}
