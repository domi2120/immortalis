use diesel::{Connection, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use std::env;

// this will run the migrations, but it wont create the database if it doesn't exist already
fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut connection = PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    let _result = connection.run_pending_migrations(MIGRATIONS);
}
