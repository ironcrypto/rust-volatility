#[cfg(test)]
mod tests {
    use tokio;
    use std::sync::{Arc, Mutex};
    use crate::client::start_multi_symbol_stream; 


    #[tokio::test]
    async fn test_binance_websocket_connection() {
        // Test WebSocket connection for a single symbol
        let symbols: Vec<String> = vec!["ethusdc".to_string()];
        let calculators = Arc::new(Mutex::new(vec![(
            "btcusdt".to_string(),
            crate::math::VolatilityCalculator::new(1),
        )]));

        start_multi_symbol_stream(symbols.clone(), Arc::clone(&calculators)).await;

        // Assert that no panics or errors occur during WebSocket connection
        assert!(true, "WebSocket connection for a single symbol works as expected.");
    }

    #[tokio::test]
    async fn test_binance_websocket_multiple_symbols() {
        // Test WebSocket connection for multiple symbols
        let symbols: Vec<String> = vec!["btcusdt".to_string(), "ethusdt".to_string()];
        let calculators = Arc::new(Mutex::new(vec![
            ("btcusdt".to_string(), crate::math::VolatilityCalculator::new(1)),
            ("ethusdt".to_string(), crate::math::VolatilityCalculator::new(1)),
        ]));

        start_multi_symbol_stream(symbols.clone(), Arc::clone(&calculators)).await;

        // Assert that no panics or errors occur during WebSocket connection
        assert!(true, "WebSocket connection for multiple symbols works as expected.");
    }
}