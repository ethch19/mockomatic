use std::str::FromStr;
use anyhow::Result;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::ConnectOptions;
use tracing_subscriber::fmt;


#[tokio::main]
async fn main() -> Result<()> {
    let chrono_fmter = tracing_subscriber::fmt::time::ChronoUtc::new("%F %T%.3f".to_string());
    let format_e = fmt::format().with_timer(chrono_fmter).with_thread_ids(true);
    tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).event_format(format_e).init();

    println!("Starting Mockomatic backend...");
    let database_url = dotenvy::var("DATABASE_URL")?;
    println!("Envrionment variable found");
    let connection_options = PgConnectOptions::from_str(&database_url).unwrap()
        .disable_statement_logging().clone();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(connection_options)
        .await?;

    sqlx::migrate!().run(&pool).await?;
    backend::http::serve(pool).await
}
