use diesel::{Connection, PgConnection};
use diesel_migrations::{
    embed_migrations, EmbeddedMigrations, HarnessWithOutput, MigrationHarness,
};
use dotenvy::dotenv;
use immortalis_backend_common::env_var_config::EnvVarConfig;
use std::sync::Arc;

// this will run the migrations, but it wont create the database if it doesn't exist already
fn main() {
    dotenv().ok();
    let env_var_config = Arc::new(envy::from_env::<EnvVarConfig>().unwrap());

    let database_url = &env_var_config.database_url;
    let mut connection = PgConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    let mut harness = HarnessWithOutput::write_to_stdout(&mut connection);
    println!("{:#?}", harness.run_pending_migrations(MIGRATIONS));
}
