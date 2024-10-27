use std::sync::Arc;

use chrono::DateTime;
use tonic::{Request, Response, Status};
use ulid::Ulid;

use crate::{
    db::{Database, DatabaseTask},
    prometheus::metrics::{CANCELLED_TASKS, SCHEDULED_TASKS},
    protos::rpc::{
        task_scheduler_server::TaskScheduler, BulkTaskRequest, BulkTaskResponse, CancelTaskRequest,
        CancelTaskResponse, GetTaskRequest, ScheduleTaskRequest, ScheduleTaskResponse, Task,
    },
};

pub struct RpcServer {
    db: Arc<Database>,
}

impl RpcServer {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl TaskScheduler for RpcServer {
    async fn schedule_task(
        &self,
        request: Request<ScheduleTaskRequest>,
    ) -> Result<Response<ScheduleTaskResponse>, Status> {
        SCHEDULED_TASKS.inc();

        let request = request.get_ref();
        let run_at = request
            .run_at
            .ok_or_else(|| Status::invalid_argument("run_at is required"))?;
        let run_at = DateTime::from_timestamp(run_at.seconds, run_at.nanos as u32)
            .ok_or_else(|| Status::invalid_argument("invalid timestamp"))?;

        let task = DatabaseTask {
            id: Ulid::new(),
            exchange: Some(request.exchange.clone()),
            routing_key: request.routing_key.clone(),
            run_at,
            payload: request.payload.clone(),
        };

        if self.db.schedule(&task).await.is_ok() {
            Ok(Response::new(ScheduleTaskResponse {
                task_id: task.id.to_bytes().to_vec(),
            }))
        } else {
            Err(Status::internal("Failed to schedule task"))
        }
    }

    async fn cancel_task(
        &self,
        request: Request<CancelTaskRequest>,
    ) -> Result<Response<CancelTaskResponse>, Status> {
        CANCELLED_TASKS.inc();
        let req = request.get_ref();
        let task_id = self.get_task_id(&req.task_id)?;

        if self.db.get_task(task_id).await.is_none() {
            return Err(Status::not_found("Task not found"));
        }

        if self.db.cancel_task(task_id).await.is_ok() {
            Ok(Response::new(CancelTaskResponse {
                task_id: task_id.to_bytes().to_vec(),
            }))
        } else {
            Err(Status::internal("Failed to cancel task"))
        }
    }

    async fn get_task(&self, request: Request<GetTaskRequest>) -> Result<Response<Task>, Status> {
        let task_id = self.get_task_id(&request.get_ref().task_id)?;
        let existing_task = self.db.get_task(task_id).await;
        if let Some(existing) = existing_task {
            Ok(Response::new(existing.into()))
        } else {
            Err(Status::not_found("Task not found"))
        }
    }

    async fn get_many_tasks(
        &self,
        request: Request<BulkTaskRequest>,
    ) -> Result<Response<BulkTaskResponse>, Status> {
        let task_ids = request
            .get_ref()
            .task_id
            .iter()
            .map(|id| self.get_task_id(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Status::invalid_argument("Invalid task_id"))?;

        let existing_tasks = self.db.get_many_tasks(&task_ids).await;
        if let Some(existing) = existing_tasks {
            Ok(Response::new(BulkTaskResponse {
                tasks: existing.into_iter().map(Into::into).collect(),
            }))
        } else {
            Err(Status::not_found("Task not found"))
        }
    }
}

impl RpcServer {
    fn get_task_id(&self, task_id: &[u8]) -> Result<Ulid, Status> {
        Ok(Ulid::from_bytes(task_id.try_into().map_err(|_| {
            Status::invalid_argument("Invalid task_id")
        })?))
    }
}
