use std::sync::LazyLock;

use metrics::INCOMING_MESSAGES;
use prometheus::Registry;
use tracing::info;
use warp::{reject::Rejection, reply::Reply, Filter};

pub static REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);

pub mod metrics {
    use std::sync::LazyLock;

    use prometheus::IntCounter;

    pub static INCOMING_MESSAGES: LazyLock<IntCounter> = LazyLock::new(|| {
        IntCounter::new("incoming_messages", "Incoming messages").expect("metric cannot be created")
    });
}

fn register_custom_metrics() {
    REGISTRY
        .register(Box::new(INCOMING_MESSAGES.clone()))
        .expect("metric cannot be registered");
}

pub async fn serve() {
    register_custom_metrics();
    let addr = ([0, 0, 0, 0], 8080);

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
