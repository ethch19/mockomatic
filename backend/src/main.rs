use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<()> {
    let chrono_fmter = tracing_subscriber::fmt::time::ChronoUtc::new("%F %T%.3f".to_string());
    let format_e = fmt::format().with_timer(chrono_fmter).with_thread_ids(true);
    tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).event_format(format_e).init();

    let database_url = dotenvy::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;
    backend::http::serve(pool).await
}
