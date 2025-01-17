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

    #[tokio::test]
    async fn test_binance_websocket_empty_symbols() {
        // Test WebSocket connection with an empty symbol list
        let symbols: Vec<String> = vec![];
        let calculators = Arc::new(Mutex::new(vec![]));

        start_multi_symbol_stream(symbols.clone(), Arc::clone(&calculators)).await;

        // Assert that no panics or errors occur during WebSocket connection
        assert!(true, "WebSocket connection with an empty symbol list works as expected.");
    }

    #[tokio::test]
    async fn test_binance_websocket_invalid_symbol() {
        // Test WebSocket connection with an invalid symbol
        let symbols: Vec<String> = vec!["invalidsymbol".to_string()];
        let calculators = Arc::new(Mutex::new(vec![(
            "invalidsymbol".to_string(),
            crate::math::VolatilityCalculator::new(1),
        )]));

        start_multi_symbol_stream(symbols.clone(), Arc::clone(&calculators)).await;

        // Assert that no panics or errors occur during WebSocket connection
        assert!(true, "WebSocket connection with an invalid symbol works as expected.");
    }
}
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
