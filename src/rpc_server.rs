use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::{
    db::Database,
    prometheus::metrics::{CANCELLED_TASKS, SCHEDULED_TASKS},
    protos::rpc::{
        task_scheduler_server::TaskScheduler, CancelTaskRequest, CancelTaskResponse,
        GetTaskRequest, GetTaskResponse, ScheduleTaskRequest, ScheduleTaskResponse,
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
        todo!();
    }

    async fn cancel_task(
        &self,
        request: Request<CancelTaskRequest>,
    ) -> Result<Response<CancelTaskResponse>, Status> {
        CANCELLED_TASKS.inc();
        todo!()
    }

    async fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> Result<Response<GetTaskResponse>, Status> {
        todo!()
    }
}
