use anyhow::Result;
use db::DatabaseTask;
use dotenvy::dotenv;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
};
use prometheus::{metrics::INCOMING_MESSAGES, serve};
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc};
use tokio::time::sleep;
use tracing::{debug, info, warn};
use ulid::Ulid;

use futures_lite::stream::StreamExt;

mod db;
mod id;
mod prometheus;
mod protos;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let result = dotenv();
    if let Err(err) = result {
        warn!("No .env file found: {}", err);
    }

    let addr = env::var("AMQP_ADDR")?;

    info!("Connecting to AMQP server at: {}", addr);

    let con = Connection::connect(&addr, ConnectionProperties::default()).await?;

    info!("Connected to AMQP server");

    let channel_1 = con.create_channel().await?;
    let channel_2 = con.create_channel().await?;

    let queue = channel_1
        .queue_declare(
            "hello",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    info!(?queue, "Declared queue");

    let mut consumer = channel_2
        .basic_consume(
            "hello",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    let handle = tokio::spawn(async move {
        while let Some(delivery) = consumer.next().await {
            let delivery = delivery.expect("Error in consumer");
            info!(?delivery, "Received message");
            INCOMING_MESSAGES.inc();
            delivery.ack(BasicAckOptions::default()).await.expect("ack");
        }
    });

    let db_url = env::var("DATABASE_URL")?;
    debug!("Connecting to database: {}", db_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    let db = Arc::new(db::Database::new(pool));

    let db2 = Arc::clone(&db);

    tokio::spawn(async move {
        info!("Spawned a new task");
        sleep(tokio::time::Duration::from_secs(5)).await;
        let _ = db
            .schedule(DatabaseTask {
                id: Ulid::new(),
                topic: "hello".to_string(),
                run_at: chrono::Utc::now() + chrono::Duration::seconds(5),
                payload: vec![1, 2, 3],
            })
            .await;
    });

    let _ = tokio::join!(handle, serve(), async move {
        db2.run().await;
    });

    Ok(())
}
