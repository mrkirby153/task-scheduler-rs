use anyhow::Result;
use dotenvy::dotenv;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
};
use std::env;
use tracing::{debug, info, warn};

use futures_lite::stream::StreamExt;

mod id;
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
            delivery.ack(BasicAckOptions::default()).await.expect("ack");
        }
    });

    handle.await?;

    // let db_url = env::var("DATABASE_URL")?;
    // debug!("Connecting to database: {}", db_url);

    // let pool = PgPoolOptions::new()
    //     .max_connections(5)
    //     .connect(&db_url)
    //     .await?;

    // let dt = Local::now();

    // let ulid: Id = Id(ulid::Ulid::new());
    // info!("Generated ULID: {}", ulid);
    // info!("As bytes: {:?}", ulid.to_bytes());

    // let bytes = [0u8; 16];

    // let query =
    //     sqlx::query("INSERT INTO tasks (id, topic, run_at, payload) VALUES ($1, $2, $3, $4)")
    //         .bind(ulid)
    //         .bind("test")
    //         .bind(dt)
    //         .bind(bytes)
    //         .execute(&pool)
    //         .await?;

    // debug!("Inserted {} rows", query.rows_affected());

    // let retrieval = sqlx::query!("SELECT * FROM tasks WHERE id = $1", &ulid.to_bytes())
    //     .fetch_one(&pool)
    //     .await?;
    // debug!("Test: {:?}", retrieval.topic);

    Ok(())
}
