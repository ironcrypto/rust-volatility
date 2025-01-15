use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{sleep, Duration};
use futures_util::StreamExt;
use log::{info, error};


pub struct BinanceClient {
    base_url: String,
    sender: UnboundedSender<(String, f64)>,
}

impl BinanceClient {
    pub async fn new(base_url: &str, sender: UnboundedSender<(String, f64)>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            base_url: base_url.to_string(),
            sender,
        })
    }
    pub async fn start_multi_symbol_stream(&self, symbols: Vec<String>) {
        for symbol in symbols {
            let sender_clone = self.sender.clone();
            let base_url = self.base_url.clone();
            
            tokio::spawn(async move {
                let mut retry_attempts = 0;

                loop {
                    match connect_to_websocket(&base_url, &symbol).await {
                        Ok(mut stream) => {
                            info!("Connected to WebSocket for symbol: {}", symbol);

                            while let Some(Ok(message)) = stream.next().await {
                                if let Ok(text) = message.to_text() {
                                    process_message(text, &symbol, &sender_clone).await;
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
                    info!(
                        "Retrying connection to {} in {} seconds...",
                        symbol, backoff.as_secs()
                    );
                    sleep(backoff).await;
                }
            });
        }
    }
}

async fn connect_to_websocket(
    base_url: &str,
    symbol: &str,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{}/{}@kline_1m", base_url, symbol);
    let (stream, _) = connect_async(&url).await?;
    Ok(stream)
}

async fn process_message(
    message: &str,
    symbol: &str,
    sender: &UnboundedSender<(String, f64)>,
) {
    match serde_json::from_str::<serde_json::Value>(message) {
        Ok(json) => {
            if let Some(close_price) = extract_close_price(&json) {
                if let Err(e) = sender.send((symbol.to_string(), close_price)) {
                    error!("Failed to send data for {}: {}", symbol, e);
                } else {
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