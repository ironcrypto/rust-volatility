mod client;
mod math;

use client::InfuraClient;
use math::VolatilityCalculator;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use tokio::{spawn, time::{self, Duration}};
use tokio::sync::mpsc;
use tracing::{info, debug, warn, error};
use tracing_subscriber::{fmt};
use tracing_appender::rolling;
use tracing_subscriber::prelude::*;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use prometheus::{Encoder, TextEncoder, GaugeVec, Registry};

const MAX_ROLLING_WINDOW_DURATION: u64 = 600_000;
const INFURA_WS_URL: &str = "wss://mainnet.infura.io/ws/v3/943fabd894044ec88ccae8613bf6b0b4";
const POOL_ADDRESS: &str = "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640";

#[tokio::main]
async fn main() {
    let file_appender = rolling::daily("logs", "volatility.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);
    let stdout = std::io::stdout();
    let (stdout_writer, _stdout_guard) = tracing_appender::non_blocking(stdout);
    
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(file_writer).with_ansi(false))
        .with(fmt::layer().with_writer(stdout_writer).with_ansi(true))
        .init();

    // Initialize Prometheus metrics
    let registry = Arc::new(Registry::new());

    // Create Prometheus GaugeVec and register it
    let volatility_gauge = prometheus::GaugeVec::new(
        prometheus::Opts::new("volatility", "ETH-USDC Volatility"),
        &["symbol"],
    ).unwrap();
    registry.register(Box::new(volatility_gauge.clone())).unwrap();
    info!("Prometheus gauge registered.");
    
    // Wrap the GaugeVec in Arc for shared access
    let volatility_gauge = Arc::new(volatility_gauge);
    
    // Shared state
    let calculator = Arc::new(tokio::sync::Mutex::new(VolatilityCalculator::new(MAX_ROLLING_WINDOW_DURATION)));
    let is_running = Arc::new(AtomicBool::new(true));

    // Channel for decoupling fetch and process
    let (tx, rx) = mpsc::unbounded_channel();

    // Create Infura client
    let client = match InfuraClient::new(INFURA_WS_URL, POOL_ADDRESS).await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to create InfuraClient: {:?}", e);
            return;
        }
    };

    // Task 1: Fetch prices
    let fetch_task = tokio::spawn(fetch_prices_task(
        Arc::clone(&is_running),
        client,
        tx,
    ));

    // Task 2: Process prices and calculate volatility
    let process_task = tokio::spawn(process_prices_task(
        rx,
        Arc::clone(&calculator),
        Arc::clone(&volatility_gauge),
    ));

    // Task 3: Start Prometheus metrics server
    let metrics_server_task = tokio::spawn(metrics_server_task(
        Arc::clone(&registry),
    ));

    // Graceful shutdown handler
    tokio::spawn({
        let is_running = Arc::clone(&is_running);
        async move {
            tokio::signal::ctrl_c().await.unwrap();
            is_running.store(false, Ordering::SeqCst);
            info!("Received shutdown signal. Stopping...");
        }
    });

    // Wait for all tasks to complete
    let _ = tokio::try_join!(fetch_task, process_task, metrics_server_task);

    info!("Shutdown complete.");
}

async fn fetch_prices_task(
    is_running: Arc<AtomicBool>,
    client: InfuraClient,
    sender: mpsc::UnboundedSender<f64>,
) {
    info!("Price fetching task started.");

    while is_running.load(Ordering::SeqCst) {
        match client.fetch_prices(&sender, 10).await {
            Ok(batch_count) => {
                info!("Fetched {} logs in this batch.", batch_count);
            }
            Err(e) => {
                error!("Error fetching prices: {:?}", e);
                warn!("Retrying in 10 seconds...");
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }
    }

    info!("Price fetching task exiting.");
}

async fn process_prices_task(
    mut receiver: mpsc::UnboundedReceiver<f64>,
    calculator: Arc<tokio::sync::Mutex<VolatilityCalculator>>,
    volatility_gauge: Arc<GaugeVec>,
) {
    while let Some(price) = receiver.recv().await {

        let mut calc = calculator.lock().await;
        calc.add_value(price); // Add price to the rolling window

        // Calculate and update volatility
        if let Some(volatility) = calc.calculate_volatility() {
            volatility_gauge
                .with_label_values(&["ETH-USDC"])
                .set(volatility); // Update Prometheus gauge
            info!("Updated Volatility: {:.6}", volatility);
        }
    }

    info!("Volatility calculation task exiting.");
}

async fn metrics_server_task(registry: Arc<Registry>) {
    let addr = ([127, 0, 0, 1], 8080).into();
    let make_svc = make_service_fn(move |_conn| {
        let registry = Arc::clone(&registry);
        async move {
            Ok::<_, hyper::Error>(service_fn(move |_req| {
                let registry = Arc::clone(&registry);
                async move {
                    let encoder = TextEncoder::new();
                    let mut buffer = Vec::new();

                    // Gather metrics from the registry
                    let metrics = registry.gather();
                    if let Err(e) = encoder.encode(&metrics, &mut buffer) {
                        error!("Failed to encode Prometheus metrics: {}", e);
                        return Ok::<_, hyper::Error>(
                            Response::builder()
                                .status(500)
                                .body(Body::from("Failed to encode metrics"))
                                .unwrap(),
                        );
                    }

                    Ok::<_, hyper::Error>(
                        Response::builder()
                            .header("Content-Type", encoder.format_type())
                            .body(Body::from(buffer))
                            .unwrap(),
                    )
                }
            }))
        }
    });

    info!("Starting Prometheus metrics server at http://127.0.0.1:8080");
    let server = Server::bind(&addr).serve(make_svc);

    // Await the server and handle errors
    if let Err(e) = server.await {
        error!("Metrics server failed: {}", e);
    }
}