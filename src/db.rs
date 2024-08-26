use std::{sync::Mutex, time::Duration};

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};
use ulid::Ulid;

pub struct Database {
    pool: Pool<Postgres>,
    token: Mutex<CancellationTokenWrwapper>,
}

struct CancellationTokenWrwapper(Option<CancellationToken>);

pub struct DatabaseTask {
    pub id: Ulid,
    pub topic: String,
    pub run_at: DateTime<Utc>,
    pub payload: Vec<u8>,
}

impl Database {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool,
            token: Mutex::new(CancellationTokenWrwapper(None)),
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

    async fn run_outstanding_tasks(&self) {}

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
            "INSERT INTO tasks (id, topic, run_at, payload) VALUES ($1, $2, $3, $4)",
            &id_bytes,
            task.topic,
            task.run_at,
            task.payload
        )
        .execute(&self.pool)
        .await;

        self.interrupt();

        Ok(())
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
                    debug!("Next run at: {:?}", run_at);
                    let now = Utc::now();
                    let delay = run_at.signed_duration_since(now).num_milliseconds();
                    if delay > 0 {
                        info!("Sleeping for {} milliseconds", delay);
                        let sleep_handle = sleep(Duration::from_millis(delay as u64));
                        let token = self.get_token();
                        select! {
                            _ = sleep_handle => {
                                debug!("Ready!");
                                self.run_outstanding_tasks().await;
                            }
                            _ = token.cancelled() => {
                                debug!("Task was canceled");
                            }
                        }
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
