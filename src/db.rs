use diesel::{
    connection::SimpleConnection,
    pg::Pg,
    prelude::*,
    r2d2,
    result::{DatabaseErrorKind, Error},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};

/// Short-hand for the database pool type to use throughout the app.
pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

pub type WebError = Box<dyn std::error::Error + Send + Sync>;

pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("./migrations/");

pub fn establish_connection() -> PgConnection {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(conn_spec);

    r2d2::Pool::builder()
        .build(manager)
        .expect("can't connect to database!")
}

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

pub fn run_migrations(
    connection: &mut impl MigrationHarness<Pg>,
    migrations: EmbeddedMigrations,
) -> Result<(), WebError> {
    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    connection.run_pending_migrations(migrations)?;

    Ok(())
}

pub fn reset_db(conn: &mut PgConnection) {
    // This command finds all tables in the current schema and truncates them.
    // 'CASCADE' ensures foreign key constraints don't stop the deletion.
    let sql = r#"
DO $$ 
DECLARE 
    r RECORD; 
BEGIN 
    -- 1. Loop through all tables in the public schema
    FOR r IN (SELECT tablename FROM pg_tables WHERE schemaname = 'public') LOOP 
        -- 2. Drop the table using CASCADE to bypass foreign key constraints
        EXECUTE 'DROP TABLE IF EXISTS ' || quote_ident(r.tablename) || ' CASCADE'; 
    END LOOP; 
END $$;
    "#;

    conn.batch_execute(sql).expect("Failed to truncate tables");
}
