use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::{spawn, time::{self, Duration}};
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use prometheus::{Encoder, TextEncoder, GaugeVec, Registry};
use tracing::{info, error};
use tracing_subscriber::{fmt};
use tracing_appender::rolling;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use crate::math::VolatilityCalculator;



mod client;
mod math;

// Duration the rolling window
const ROLLING_WINDOW_DURATION: u64 = u64::from_be(30);
const INTERVAL_DURATION: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() {
    println!("HELLO ARRAKIS! THIS IS MY RUST PROGRAM!");

    // Step 1: Initialize logging (both terminal and file)
    let file_appender = rolling::daily("logs", "volatility.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let stdout = std::io::stdout();
    let (stdout_writer, _stdout_guard) = tracing_appender::non_blocking(stdout);

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false);
    let stdout_layer = fmt::layer()
        .with_writer(stdout_writer)
        .with_ansi(true);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer)
        .init();

    info!("Starting Binance WebSocket Volatility Estimator");
    println!("Starting Binance Volatility Estimator. Press Ctrl+C to exit.");

    // Initialize Prometheus metrics
    let registry = Arc::new(Registry::new());
    let volatility_gauge = GaugeVec::new(
        prometheus::Opts::new("volatility", "Volatility metrics for symbols"),
        &["symbol"]
    ).unwrap();
    registry.register(Box::new(volatility_gauge.clone())).unwrap();
    info!("Volatility gauge registered successfully with the Prometheus registry.");



    // Step 2: Symbols and shared state
    let symbols: Vec<String> = vec!["ethusdc".to_string()];

    // Shared volatility calculators for each symbol
    let calculators: Arc<Mutex<Vec<(String, VolatilityCalculator)>>> = Arc::new(Mutex::new(
        symbols
            .iter()
            .map(|symbol| (symbol.clone(), VolatilityCalculator::new(ROLLING_WINDOW_DURATION))) 
            .collect(),
    ));


    // Signal for graceful shutdown
    let is_running = Arc::new(AtomicBool::new(true));
    let is_running_clone = Arc::clone(&is_running);

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        is_running_clone.store(false, Ordering::SeqCst);
        info!("Received shutdown signal. Stopping...");
    });

    // Step 3: Start WebSocket streams
    let calculators_ws = Arc::clone(&calculators);
    let volatility_gauge_ws = volatility_gauge.clone();
    let symbols_ws = symbols.clone();

    // Spawn a task to handle WebSocket streaming for OHLC data
    let public_stream_task = spawn(async move {
        client::start_multi_symbol_stream(symbols_ws, calculators_ws).await;
    });

    // Step 4: Periodic calculation logic
    let calculators_calc = Arc::clone(&calculators);
    let is_running_clone_for_calc = Arc::clone(&is_running);
    let registry_calc = Arc::clone(&registry);
    
    let volatility_calc_task = spawn(async move {
        let mut interval = time::interval(INTERVAL_DURATION); 
        while is_running_clone_for_calc.load(Ordering::SeqCst) {
            interval.tick().await;
            
            // Attempt to acquire the lock
            let calculators_lock = match calculators_calc.lock() {
                Ok(lock) => lock,
                Err(e) => {
                    error!("Failed to acquire lock: {}", e);
                    continue; // Skip this iteration and try again
                }
            };

            // Calculate and log volatility for each symbol
            for (symbol, calculator) in calculators_lock.iter() {
                if let Some(volatility) = calculator.calculate_volatility() {
                    volatility_gauge_ws.with_label_values(&[symbol]).set(volatility);
                    info!("Volatility Gauge updated for {}: {:.6}", symbol, volatility);

                    // Debug log metrics
                    let gathered_metrics = registry_calc.gather();
                    let encoder = TextEncoder::new();
                    let mut buffer = Vec::new();
                    if let Err(e) = encoder.encode(&gathered_metrics, &mut buffer) {
                        error!("Failed to encode metrics in calculation task: {}", e);
                    } else {
                        info!("Updated metrics during calculation task:\n{}", String::from_utf8_lossy(&buffer));
                    }
                } else {
                    info!("{}: Not enough data for volatility calculation", symbol);
                }
            }
        }
    });

    // Prometheus metrics server task
    let is_running_clone_metrics = Arc::clone(&is_running);
    let registry_metrics = Arc::clone(&registry);
    let metrics_server_task = spawn(async move {
        let make_svc = make_service_fn(move |_conn| {
            let registry = Arc::clone(&registry_metrics);
            async move {
                Ok::<_, hyper::Error>(service_fn(move |_: Request<Body>| {
                    let encoder = TextEncoder::new();
                    let mut buffer = Vec::new();
        
                    // Gather and log metrics
                    let gathered_metrics = registry.gather();
                    for metric_family in &gathered_metrics {
                        info!("Serving metric: {:?}", metric_family.get_name());
                    }
                    if let Err(e) = encoder.encode(&gathered_metrics, &mut buffer) {
                        error!("Failed to encode metrics: {}", e);
                    } else {
                        info!("Serving metrics:\n{}", String::from_utf8_lossy(&buffer));
                    }
        
                    // Build response with appropriate Content-Type header
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
            _ = server => {
                error!("Metrics server error");
            }
            _ = async {
                while is_running_clone_metrics.load(Ordering::SeqCst) {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            } => {
                info!("Shutting down metrics server...");
            }
        }
    });

    // Keep the main task alive until shutdown
    while is_running.load(Ordering::SeqCst) {
        time::sleep(Duration::from_secs(1)).await;
    }

    info!("Shutting down...");
    let _ = tokio::try_join!(public_stream_task, volatility_calc_task, metrics_server_task);
    info!("Shutdown complete.");
}