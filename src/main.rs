use actix_web::{web, App, HttpServer};

use handlebars::{DirectorySourceOptions, Handlebars};
use rust_forum::{
    establish_connection,
    routes::{
        index::index,
        users::{users_login, users_login_post, users_register, users_register_post},
    },
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let host = "127.0.0.1";
    let port = 3000;

    println!("listening at http://{}:{}", host, port);

    HttpServer::new(|| {
        // let db = initialize_db_pool();
        let db = establish_connection();
        let db_ref = web::Data::new(db);

        let mut handlebars_options = DirectorySourceOptions::default();
        handlebars_options.tpl_extension = ".hbs".to_owned();
        handlebars_options.hidden = false;
        handlebars_options.temporary = false;

        let mut handlebars = Handlebars::new();
        handlebars
            .register_templates_directory("./templates", handlebars_options)
            .unwrap();
        let handlebars_ref = web::Data::new(handlebars);

        let users_scope = web::scope("/users")
            .service(users_login)
            .service(users_login_post)
            .service(users_register)
            .service(users_register_post);

        App::new()
            .app_data(db_ref)
            .app_data(handlebars_ref)
            .service(users_scope)
            .service(index)
        // .service(echo)
        // .route("/hey", web::get().to(hey))
    })
    .bind((host, port))?
    .run()
    .await
}
