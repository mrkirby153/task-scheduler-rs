use std::sync::LazyLock;

use metrics::{BULK_TASK_REQUESTS, SCHEDULED_TASKS};
use prometheus::Registry;
use tracing::{debug, info};
use warp::{reject::Rejection, reply::Reply, Filter};

pub static REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);

pub mod metrics {
    use std::sync::LazyLock;

    use prometheus::{IntCounter, IntGauge};

    pub static SCHEDULED_TASKS: LazyLock<IntCounter> = LazyLock::new(|| {
        IntCounter::new("scheduled_tasks", "Scheduled tasks").expect("metric cannot be created")
    });

    pub static CANCELLED_TASKS: LazyLock<IntCounter> = LazyLock::new(|| {
        IntCounter::new("cancelled_tasks", "Cancelled tasks").expect("metric cannot be created")
    });

    pub static GET_TASKS: LazyLock<IntCounter> = LazyLock::new(|| {
        IntCounter::new("get_tasks", "Get tasks").expect("metric cannot be created")
    });

    pub static PROCESSED_TASKS: LazyLock<IntCounter> = LazyLock::new(|| {
        IntCounter::new("processed_tasks", "Tasks delivered to RabbitMQ")
            .expect("metric cannot be created")
    });

    pub static SUCCESSFUL_TASKS: LazyLock<IntCounter> = LazyLock::new(|| {
        IntCounter::new("successful_tasks", "Successful tasks").expect("metric cannot be created")
    });

    pub static FAILED_TASKS: LazyLock<IntCounter> = LazyLock::new(|| {
        IntCounter::new("failed_tasks", "Failed tasks").expect("metric cannot be created")
    });

    pub static TOTAL_TASKS: LazyLock<IntGauge> = LazyLock::new(|| {
        IntGauge::new("total_tasks", "Total tasks").expect("metric cannot be created")
    });

    pub static BULK_TASK_REQUESTS: LazyLock<IntGauge> = LazyLock::new(|| {
        IntGauge::new("bulk_task_requests", "Bulk task requests").expect("metric cannot be created")
    });
}

fn register_custom_metrics() {
    REGISTRY
        .register(Box::new(SCHEDULED_TASKS.clone()))
        .expect("metric cannot be registered");
    REGISTRY
        .register(Box::new(metrics::CANCELLED_TASKS.clone()))
        .expect("metric cannot be registered");
    REGISTRY
        .register(Box::new(metrics::PROCESSED_TASKS.clone()))
        .expect("metric cannot be registered");
    REGISTRY
        .register(Box::new(metrics::GET_TASKS.clone()))
        .expect("metric cannot be registered");
    REGISTRY
        .register(Box::new(metrics::TOTAL_TASKS.clone()))
        .expect("metric cannot be registered");
    REGISTRY
        .register(Box::new(BULK_TASK_REQUESTS.clone()))
        .expect("metric cannot be registered");

    debug!("Registered custom metrics");
}

pub async fn serve() {
    register_custom_metrics();
    let addr = ([0, 0, 0, 0], 8081);

    info!("Starting metrics server at: {:?}", addr);

    let hello = warp::path!("_metrics").and_then(metrics_handler);

    warp::serve(hello).run(addr).await;
}

async fn metrics_handler() -> Result<impl Reply, Rejection> {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();

    let mut buff = Vec::new();

    if let Err(e) = encoder.encode(&REGISTRY.gather(), &mut buff) {
        eprintln!("Error encoding metrics: {}", e);
        return Err(warp::reject::reject());
    };

    let mut res = match String::from_utf8(buff) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error converting metrics to string: {}", e);
            return Err(warp::reject::reject());
        }
    };

    let mut buff = Vec::new();
    if let Err(e) = encoder.encode(&prometheus::gather(), &mut buff) {
        eprintln!("Error encoding metrics: {}", e);
        return Err(warp::reject::reject());
    };

    if let Ok(v) = String::from_utf8(buff) {
        res.push_str(&v);
    }

    Ok(res)
}
