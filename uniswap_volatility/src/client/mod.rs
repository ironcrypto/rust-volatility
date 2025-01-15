use web3::transports::WebSocket;
use web3::types::{Log, Address, FilterBuilder};
use web3::Web3;
use web3::ethabi::{decode, ParamType, Token};
use primitive_types::U256;
use futures_util::StreamExt;
use tracing::{info, error, warn};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub struct InfuraClient {
    web3: Web3<WebSocket>,
    pool_address: Address,
}

impl InfuraClient {
    /// Creates a new client connected to the given Infura WebSocket URL.
    pub async fn new(infura_ws_url: &str, pool_address: &str) -> web3::Result<Self> {
        let ws = WebSocket::new(infura_ws_url).await?;
        let web3 = Web3::new(ws);

        Ok(Self {
            web3,
            pool_address: pool_address.parse().unwrap(),
        })
    }

    /// Decodes the price from the Uniswap log event.
    fn decode_price(&self, log: Log) -> Option<f64> {
        let decoded_data = decode(
            &[
                ParamType::Int(256),  // amount0
                ParamType::Int(256),  // amount1
                ParamType::Uint(160), // sqrtPriceX96
                ParamType::Uint(128), // liquidity
                ParamType::Int(24),   // tick
            ],
            &log.data.0,
        )
        .ok()?;

        let sqrt_price_x96 = match &decoded_data[2] {
            Token::Uint(value) => U256::from(value.as_u128()),
            _ => return None,
        };

        let price = (sqrt_price_x96.as_u128() as f64 / 2f64.powi(96)).powi(2);
        let dollar_price = 10f64.powi(12) / price;

        Some(dollar_price)
    }

    /// Fetches prices from the WebSocket and sends them through a channel.
    pub async fn fetch_prices(
        &self,
        sender: &UnboundedSender<f64>,
        max_logs_per_batch: usize,
    ) -> web3::Result<usize> {
        let filter = FilterBuilder::default()
            .address(vec![self.pool_address])
            .topics(None, None, None, None)
            .build();

        let mut logs = self.web3.eth_subscribe().subscribe_logs(filter).await?;
        info!("Listening for price updates...");

        let mut processed_count = 0; // Count of logs processed in this batch

        while let Some(log) = logs.next().await {
            match log {
                Ok(log) => {
                    if let Some(price) = self.decode_price(log) {
                        info!("New Price: {}", price);

                        // Send the price to the channel
                        if let Err(e) = sender.send(price) {
                            warn!("Failed to send price: {:?}", e);
                            break; // Stop processing if the channel is closed
                        }

                        processed_count += 1;

                        // Exit after processing a batch
                        if processed_count >= max_logs_per_batch {
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading log: {:?}", e);
                }
            }
        }

        Ok(processed_count)
    }
}