use std::env;

use anyhow::Result;
use chrono::Local;
use dotenvy::dotenv;
use id::Id;
use sqlx::postgres::PgPoolOptions;
use tracing::{debug, info, warn};

mod id;
mod protos;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let result = dotenv();
    if let Err(err) = result {
        warn!("No .env file found: {}", err);
    }

    let db_url = env::var("DATABASE_URL")?;
    debug!("Connecting to database: {}", db_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    let dt = Local::now();

    let ulid: Id = Id(ulid::Ulid::new());
    info!("Generated ULID: {}", ulid);
    info!("As bytes: {:?}", ulid.to_bytes());

    let bytes = [0u8; 16];

    let query =
        sqlx::query("INSERT INTO tasks (id, topic, run_at, payload) VALUES ($1, $2, $3, $4)")
            .bind(ulid)
            .bind("test")
            .bind(dt)
            .bind(bytes)
            .execute(&pool)
            .await?;

    debug!("Inserted {} rows", query.rows_affected());

    let retrieval = sqlx::query!("SELECT * FROM tasks WHERE id = $1", &ulid.to_bytes())
        .fetch_one(&pool)
        .await?;
    debug!("Test: {:?}", retrieval.topic);

    Ok(())
}
