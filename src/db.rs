use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use ulid::Ulid;

use crate::{amqp::Amqp, id::Id};

pub struct Database {
    pool: Pool<Postgres>,
    amqp: Arc<Amqp>,
    token: Mutex<CancellationTokenInner>,
}

struct CancellationTokenInner(Option<CancellationToken>);

pub struct DatabaseTask {
    pub id: Ulid,
    pub exchange: Option<String>,
    pub routing_key: String,
    pub run_at: DateTime<Utc>,
    pub payload: Vec<u8>,
}

struct DatabaseTaskTransport {
    id: Vec<u8>,
    exchange: Option<String>,
    routing_key: String,
    run_at: DateTime<Utc>,
    payload: Vec<u8>,
}

impl Database {
    pub fn new(pool: Pool<Postgres>, amqp: Arc<Amqp>) -> Self {
        Self {
            pool,
            amqp,
            token: Mutex::new(CancellationTokenInner(None)),
        }
    }

    async fn get_next_run_at(&self) -> Option<DateTime<Utc>> {
        let query = sqlx::query!("SELECT run_at FROM tasks ORDER BY run_at ASC LIMIT 1")
            .fetch_optional(&self.pool)
            .await;
        if let Ok(Some(row)) = query {
            Some(row.run_at)
        } else {
            None
        }
    }

    async fn run_outstanding_tasks(&self) -> Result<i32> {
        debug!("Running outstanding tasks");
        let tasks = self.get_outstanding_tasks(None).await?;

        let by_topic = tasks.iter().fold(
            HashMap::new(),
            |mut acc: HashMap<&String, Vec<&DatabaseTask>>, task| {
                acc.entry(&task.routing_key).or_default().push(task);
                acc
            },
        );

        debug!(
            "Found {} tasks across {} topics",
            tasks.len(),
            by_topic.len(),
        );

        let mut published_tasks = 0;

        let mut tx = self.pool.begin().await?;
        for (topic, tasks) in by_topic {
            debug!("Running tasks for topic: {}", topic);
            for task in tasks {
                debug!("Running task: {}", task.id);
                if let Err(e) = self.amqp.publish(task).await {
                    error!("Failed to publish task: {}", e);
                }

                sqlx::query("DELETE FROM tasks WHERE id = $1")
                    .bind(Id(task.id))
                    .execute(&mut *tx)
                    .await?;
                published_tasks += 1;
            }
        }
        debug!("Published {} tasks", published_tasks);
        tx.commit().await?;
        Ok(published_tasks)
    }

    fn get_token(&self) -> CancellationToken {
        let mut token = self.token.lock().unwrap();
        if token.0.is_none() {
            token.0 = Some(CancellationToken::new());
        }
        token.0.as_ref().unwrap().clone()
    }

    pub async fn schedule(&self, task: DatabaseTask) -> Result<()> {
        let id_bytes = task.id.to_bytes();
        let _ = sqlx::query!(
            "INSERT INTO tasks (id, exchange, routing_key, run_at, payload) VALUES ($1, $2, $3, $4, $5)",
            &id_bytes,
            task.exchange,
            task.routing_key,
            task.run_at,
            task.payload
        )
        .execute(&self.pool)
        .await;

        self.interrupt();

        Ok(())
    }

    pub async fn get_scheduled_task_count(&self) -> Result<i64> {
        let count = sqlx::query!("SELECT COUNT(id) FROM tasks")
            .fetch_one(&self.pool)
            .await?;
        Ok(count.count.unwrap_or(0))
    }

    async fn get_outstanding_tasks(
        &self,
        run_at: Option<DateTime<Utc>>,
    ) -> Result<Vec<DatabaseTask>> {
        let run_at = run_at.unwrap_or_else(Utc::now);
        debug!("Getting oustanding tasks since {:?}", run_at);
        let transport = sqlx::query_as!(
            DatabaseTaskTransport,
            "SELECT id, exchange, routing_key, run_at, payload FROM tasks WHERE run_at <= $1 ORDER BY run_at ASC",
            run_at
        )
        .fetch_all(&self.pool)
        .await?;

        let tasks = transport
            .into_iter()
            .map(|t| t.try_into())
            .filter_map(Result::ok)
            .collect();

        Ok(tasks)
    }

    fn interrupt(&self) {
        if let Some(token) = self.token.lock().unwrap().0.take() {
            debug!("Canceling the previous task");
            token.cancel();
        }
    }

    pub async fn run(&self) {
        loop {
            let next_run_at = self.get_next_run_at().await;
            match next_run_at {
                Some(run_at) => {
                    let now = Utc::now();
                    let delay = run_at.signed_duration_since(now).num_milliseconds();
                    if delay > 0 {
                        info!("Next task runs at {}", run_at);
                        let sleep_handle = sleep(Duration::from_millis(delay as u64));
                        let token = self.get_token();
                        select! {
                            _ = sleep_handle => {
                                let _ = self.run_outstanding_tasks().await;
                            }
                            _ = token.cancelled() => {
                            }
                        }
                    } else {
                        warn!("Missed a task by {}ms", -delay);
                        let _ = self.run_outstanding_tasks().await;
                    }
                }
                None => {
                    debug!("No tasks found");
                    self.get_token().cancelled().await;
                }
            }
        }
    }
}

impl TryInto<DatabaseTask> for DatabaseTaskTransport {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<DatabaseTask> {
        let bytes = self.id.as_slice();
        Ok(DatabaseTask {
            id: Ulid::from_bytes(bytes.try_into()?),
            routing_key: self.routing_key,
            exchange: self.exchange,
            run_at: self.run_at,
            payload: self.payload,
        })
    }
}
