use amqp::Amqp;
use anyhow::Result;
use db::Database;
use dotenvy::dotenv;

use prometheus::{metrics::TOTAL_TASKS, serve};
use protos::rpc::task_scheduler_server::TaskSchedulerServer;
use rpc_server::RpcServer;
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc, time::Duration};
use tokio::time::sleep;
use tonic::transport::Server;
use tracing::{debug, info, warn};

mod amqp;
mod db;
mod id;
mod prometheus;
mod protos;
mod rpc_server;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let result = dotenv();
    if let Err(err) = result {
        warn!("No .env file found: {}", err);
    }

    let addr = env::var("AMQP_ADDR")?;

    let amqp = Arc::new(Amqp::new(&addr).await?);

    let db_url = env::var("DATABASE_URL")?;
    debug!("Connecting to database: {}", db_url);

    let pool_size = env::var("DATABASE_POOL_SIZE")
        .unwrap_or("5".to_string())
        .parse::<u32>()?;

    let pool = PgPoolOptions::new()
        .max_connections(pool_size)
        .connect(&db_url)
        .await?;

    let db = Arc::new(Database::new(pool, amqp.clone()));

    let server_address = env::var("GRPC_SERVER_ADDRESS").unwrap_or("[::1]:50051".to_string());

    info!("Starting gRPC server at: {}", server_address);

    let rpc_server = RpcServer::new(db.clone());
    let server = Server::builder()
        .add_service(TaskSchedulerServer::new(rpc_server))
        .serve(server_address.parse()?);

    let statistics = collect_statistics(db.clone());

    info!("Ready!");
    let _ = tokio::join!(
        serve(),
        async move {
            db.run().await;
        },
        server,
        statistics
    );

    Ok(())
}

async fn collect_statistics(db: Arc<Database>) {
    loop {
        if let Ok(count) = db.get_scheduled_task_count().await {
            TOTAL_TASKS.set(count);
        }

        sleep(Duration::from_secs(30)).await;
    }
}
