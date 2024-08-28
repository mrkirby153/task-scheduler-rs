use anyhow::Result;
use lapin::{Channel, Connection, ConnectionProperties};
use tracing::{debug, error, info};

use crate::{
    db::DatabaseTask,
    prometheus::metrics::{FAILED_TASKS, SUCCESSFUL_TASKS},
};

pub struct Amqp {
    connection: Connection,
    channel: Channel,
}

impl Amqp {
    pub async fn new(addr: &str) -> Result<Self> {
        info!("Connecting to AMQP server at {}", addr);
        let con = Connection::connect(addr, ConnectionProperties::default()).await?;
        let channel = con.create_channel().await?;
        Ok(Self {
            connection: con,
            channel,
        })
    }

    pub async fn publish(&self, message: &DatabaseTask) -> Result<()> {
        debug!(
            "Publishing message to exchange {:?} with routing key {}",
            message.exchange, message.routing_key
        );

        let result = self
            .channel
            .basic_publish(
                message.exchange.as_deref().unwrap_or(""),
                &message.routing_key,
                Default::default(),
                &message.payload[..],
                Default::default(),
            )
            .await;
        match result {
            Ok(_) => {
                SUCCESSFUL_TASKS.inc();
            }
            Err(err) => {
                FAILED_TASKS.inc();
                error!("Failed to publish message: {:?}", err);
                return Err(err.into());
            }
        }

        debug!("Published message with result {:?}", result);
        Ok(())
    }
}
