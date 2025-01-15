mod client;
mod math;

use client::BinanceClient;
use math::VolatilityCalculator;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;
use tokio::sync::mpsc;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use prometheus::{Encoder, TextEncoder, GaugeVec, Registry};
use tracing::{info, debug, error};
use tracing_subscriber::fmt;
use tracing_appender::rolling;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;


const MAX_ROLLING_WINDOW_DURATION: u64 = u64::from_be(30);
const BINANCE_WS_URL: &str = "wss://stream.binance.com:9443/ws";
const SYMBOL: &str = "ethusdc";

#[tokio::main]
async fn main() {
    // Initialize logging (both terminal and file)
    
    let file_appender = rolling::daily("logs", "volatility.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);
    let stdout = std::io::stdout();
    let (stdout_writer, _stdout_guard) = tracing_appender::non_blocking(stdout);

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(file_writer).with_ansi(false))
        .with(fmt::layer().with_writer(stdout_writer).with_ansi(true))
        .init();
    info!("Logging initialized.");

    info!("Starting Binance WebSocket Volatility Estimator");
    eprintln!("Starting Binance Volatility Estimator. Press Ctrl+C to exit.");

    // // Initialize Prometheus metrics
    let (volatility_gauge, registry) = init_metrics();

    // Channel for decoupling fetch and process
    let (tx, rx) = mpsc::unbounded_channel();
    println!("Channel for decoupling fetch and process created.");
    
    // Create Infura client
    debug!("Creating BinanceClient...");
    let client = match BinanceClient::new(BINANCE_WS_URL, tx).await{
        Ok(client) => client,
        Err(e) => {
            error!("Failed to create BinanceClient: {:?}", e);
            return;
        }
    };
  
    // Symbols and shared state
    let symbols: Vec<String> = vec![SYMBOL.to_string()];
    let calculators = Arc::new(Mutex::new(
        symbols.iter()
            .map(|symbol| (symbol.clone(), VolatilityCalculator::new(MAX_ROLLING_WINDOW_DURATION)))
            .collect(),
    ));
    let is_running = Arc::new(AtomicBool::new(true));

    // Task 1: WebSocket stream 
    let websocket_task = tokio::spawn(start_websocket_task(
        symbols.clone(),
        client,
        Arc::clone(&is_running),
    ));

    // Task 2: volatility calculation 
    let calc_task = tokio::spawn(start_volatility_calc_task(
        rx,
        Arc::clone(&calculators),
        Arc::clone(&volatility_gauge),
    ));


    // Task 3: Prometheus metrics server
    let metrics_task = tokio::spawn(start_metrics_server(
        registry,
        Arc::clone(&is_running),
    ));

    tokio::select! {
        _ = websocket_task => info!("WebSocket task exited."),
        _ = calc_task => info!("Calculation task exited."),
        _ = metrics_task => info!("Metrics task exited."),
        _ = handle_shutdown_signal(Arc::clone(&is_running)) => info!("Shutdown signal received."),
    }
    info!("All tasks completed or shutdown signal processed.");

}

// Initialize Prometheus metrics
fn init_metrics() -> (Arc<GaugeVec>, Arc<Registry>) {
    // Create a new Prometheus registry
    let registry = Arc::new(Registry::new());

    // Create a new GaugeVec for volatility metrics
    let volatility_gauge = GaugeVec::new(
        prometheus::Opts::new("binance_volatility", "Volatility metrics for Binance symbols"),
        &["symbol"],
    ).unwrap();

    // Register the GaugeVec with the registry
    registry.register(Box::new(volatility_gauge.clone())).unwrap();
    info!("Volatility gauge registered successfully with Prometheus.");

    // Wrap the GaugeVec in Arc for shared ownership and return
    (Arc::new(volatility_gauge), registry)
}

// WebSocket stream task
async fn start_websocket_task(
    symbols: Vec<String>,
    client: BinanceClient,
    is_running: Arc<AtomicBool>,
) {
    while is_running.load(Ordering::SeqCst) {
        client.start_multi_symbol_stream(symbols.clone()).await;
        tokio::time::sleep(Duration::from_secs(1)).await; // Retry on failure
    }
}


// Volatility calculation task
async fn start_volatility_calc_task(
    mut receiver: mpsc::UnboundedReceiver<(String, f64)>,
    calculators: Arc<Mutex<Vec<(String, VolatilityCalculator)>>>,
    volatility_gauge: Arc<GaugeVec>,
) {
    while let Some((symbol, price)) = receiver.recv().await {
        let mut calculators_lock = match calculators.lock() {
            Ok(lock) => lock,
            Err(e) => {
                error!("Failed to acquire lock: {}", e);
                continue;
            }
        };

        if let Some((_, calculator)) = calculators_lock.iter_mut().find(|(s, _)| s == &symbol) {
            calculator.add_value(price);
            if let Some(volatility) = calculator.calculate_volatility() {
                volatility_gauge
                    .with_label_values(&[&symbol])
                    .set(volatility);
                info!("Volatility Gauge updated for {}: {:.6}", symbol, volatility);
            } else {
                info!("{}: Not enough data for volatility calculation", symbol);
            }
        }
    }

    info!("Volatility calculation task exiting.");
}

// Prometheus metrics server task
async fn start_metrics_server(
    registry: Arc<Registry>,
    is_running: Arc<AtomicBool>,
) {
    let make_svc = make_service_fn(move |_conn| {
        let registry = Arc::clone(&registry);
        async move {
            Ok::<_, hyper::Error>(service_fn(move |_: Request<Body>| {
                let encoder = TextEncoder::new();
                let mut buffer = Vec::new();

                let gathered_metrics = registry.gather();
                if encoder.encode(&gathered_metrics, &mut buffer).is_err() {
                    error!("Failed to encode metrics.");
                }

                async move {
                    Ok::<_, hyper::Error>(
                        Response::builder()
                            .header("Content-Type", "text/plain; version=0.0.4")
                            .body(Body::from(buffer))
                            .unwrap(),
                    )
                }
            }))
        }
    });

    let addr = ([127, 0, 0, 1], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);

    tokio::select! {
        _ = server => error!("Metrics server error."),
        _ = async {
            while is_running.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        } => info!("Shutting down metrics server..."),
    }
}

// Handle shutdown signal
async fn handle_shutdown_signal(is_running: Arc<AtomicBool>) {
    tokio::signal::ctrl_c().await.unwrap();
    is_running.store(false, Ordering::SeqCst);
    info!("Received shutdown signal. Stopping...");
}