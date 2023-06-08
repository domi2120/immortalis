use diesel::{Connection, PgConnection};
use diesel_migrations::{
    embed_migrations, EmbeddedMigrations, HarnessWithOutput, MigrationHarness,
};
use dotenvy::dotenv;
use immortalis_backend_common::env_var_config::EnvVarConfigCommon;
use std::{sync::Arc, thread, time::Duration};
use tracing::{error, info};

// this will run the migrations, but it wont create the database if it doesn't exist already
fn main() -> Result<(), ()> {
    dotenv().ok();
    let env_var_config = Arc::new(envy::from_env::<EnvVarConfigCommon>().unwrap());
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .event_format(tracing_subscriber::fmt::format::json())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    const MAX_ATTEMPTS: usize = 20;
    const BACKOFF_DURATION_SECONDS: u64 = 10;

    for _i in 0..MAX_ATTEMPTS {
        let mut connection =
            match PgConnection::establish(&env_var_config.general_config.database_url) {
                Ok(c) => c,
                Err(e) => {
                    error!(
                        "Error connecting to Database, retrying in {} seconds. Error was: {}",
                        BACKOFF_DURATION_SECONDS, e
                    );
                    thread::sleep(Duration::from_secs(BACKOFF_DURATION_SECONDS));
                    continue;
                }
            };

        const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

        let mut harness = HarnessWithOutput::write_to_stdout(&mut connection);
        info!("{:#?}", harness.run_pending_migrations(MIGRATIONS));
        return Ok(());
    }
    error!(
        "Stopped retrying after {} attempts with {} seconds between each",
        MAX_ATTEMPTS, BACKOFF_DURATION_SECONDS
    );
    Err(())
}
