pub mod models;
pub mod schema;

pub mod routes;

pub mod comments;
pub mod posts;
pub mod users;
pub mod db;
pub mod utils;

use diesel::{Connection, PgConnection};
use dotenvy::dotenv;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
