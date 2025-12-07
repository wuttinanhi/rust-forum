use dotenv::dotenv;
use rust_forum::db::{establish_connection, reset_db, run_migrations};

fn main() {
    dotenv().ok();

    let mut conn = establish_connection();

    // reset db
    reset_db(&mut conn);

    // run migrations
    run_migrations(&mut conn, rust_forum::db::MIGRATIONS).expect("failed to run migrations");

    println!("reset database done");
}
