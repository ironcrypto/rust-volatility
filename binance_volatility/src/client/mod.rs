#[cfg(test)]
mod tests;

use crate::math::VolatilityCalculator;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio::net::TcpStream;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use log::{info, error};
use futures_util::StreamExt;



pub async fn start_multi_symbol_stream(
    symbols: Vec<String>,
    calculators: Arc<Mutex<Vec<(String, VolatilityCalculator)>>>,
) {
    for symbol in symbols {
        let calculators_clone = Arc::clone(&calculators);
        tokio::spawn(async move {
            let mut retry_attempts = 0;
            loop {
                match connect_to_websocket(&symbol).await {
                    Ok(mut stream) => {
                        info!("Connected to WebSocket for symbol: {}", symbol);

                        while let Some(Ok(message)) = stream.next().await {
                            if let Ok(text) = message.to_text() {
                                process_message(text, &symbol, &calculators_clone).await;
                            }
                        }

                        error!("WebSocket connection for {} closed unexpectedly. Reconnecting...", symbol);
                    }
                    Err(e) => {
                        retry_attempts += 1;
                        error!(
                            "Failed to connect to WebSocket for {} (attempt {}): {}",
                            symbol, retry_attempts, e
                        );
                    }
                }

                // Backoff between retries
                let backoff = Duration::from_secs(retry_attempts.min(10) as u64);
                info!("Retrying connection to {} in {} seconds...", symbol, backoff.as_secs());
                sleep(backoff).await;
            }
        });
    }
}

async fn connect_to_websocket(symbol: &str) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("wss://stream.binance.com:9443/ws/{}@kline_1m", symbol);
    let (stream, _) = connect_async(&url).await?;
    Ok(stream)
}

async fn process_message(
    message: &str,
    symbol: &str,
    calculators: &Arc<Mutex<Vec<(String, VolatilityCalculator)>>>,
) {
    match serde_json::from_str::<serde_json::Value>(message) {
        Ok(json) => {
            if let Some(close_price) = extract_close_price(&json) {
                let mut calculators_lock = calculators.lock().unwrap();
                if let Some((_, calculator)) = calculators_lock.iter_mut().find(|(s, _)| s == symbol) {
                    calculator.add_value(close_price);
                    info!("Streamed Data [{}]: Close price = {}", symbol, close_price);
                }
            }
        }
        Err(e) => {
            error!("Failed to parse WebSocket message for {}: {}", symbol, e);
        }
    }
}

fn extract_close_price(json: &serde_json::Value) -> Option<f64> {
    json["k"]["c"].as_str()?.parse::<f64>().ok()
}



