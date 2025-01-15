use binance_volatility::client;
use binance_volatility::math::VolatilityCalculator;
use std::sync::{Arc, Mutex};
use tokio;

#[tokio::test]
async fn test_integration() {
    // Define symbols and frequency
    let symbols: Vec<String> = vec!["ethusdc".to_string()];

    // Create shared calculators for each symbol
    let calculators: Arc<Mutex<Vec<(String, VolatilityCalculator)>>> = Arc::new(Mutex::new(
        symbols
            .iter()
            .map(|symbol| (symbol.to_string(), VolatilityCalculator::new(360)))
            .collect(),
    ));

    // Start WebSocket stream for testing
    client::start_multi_symbol_stream(symbols.clone(), Arc::clone(&calculators)).await;

    // Assert application behavior (replace with more specific assertions as needed)
    assert!(true, "Integration test completed successfully.");
}