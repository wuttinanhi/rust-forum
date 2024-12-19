use diesel::{
    prelude::*,
    r2d2,
    result::{DatabaseErrorKind, Error},
};

/// Short-hand for the database pool type to use throughout the app.
pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

pub type DbError = Box<dyn std::error::Error + Send + Sync>;

// pub fn initialize_db_pool() -> DbPool {
//     let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
//     let manager = r2d2::ConnectionManager::<PgConnection>::new(conn_spec);

//     r2d2::Pool::builder()
//         .build(manager)
//         .expect("database URL should be valid path to SQLite DB file")
// }

pub fn map_diesel_error_to_message(error: diesel::result::Error) -> &'static str {
    match error {
        Error::DatabaseError(kind, _) => match kind {
            DatabaseErrorKind::UniqueViolation => "Duplicate entry: the value already exists.",
            DatabaseErrorKind::ForeignKeyViolation => {
                "Foreign key violation: related data is missing."
            }
            _ => "A database error occurred.",
        },
        Error::NotFound => "The requested item was not found.",
        Error::QueryBuilderError(_) => "There was an issue building the database query.",
        Error::RollbackTransaction => "Transaction was rolled back.",
        _ => "An unexpected database error occurred.",
    }
}
