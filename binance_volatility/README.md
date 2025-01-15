
# Binance Volatility Estimator

This Rust project implements a Binance WebSocket client to retrieve OHLC data and calculates rolling window volatility based on a given algorithm.

## Structure

src/
├── client/                # Contains WebSocket client logic
│   ├── mod.rs             # Module entry point
│   └── tests.rs           # Unit tests for `client` module
├── math/                  # Contains mathematical logic
│   ├── mod.rs             # Module entry point
│   └── tests.rs           # Unit tests for `math` module
├── main.rs                # Application entry point
tests/                     # Integration tests
├── integration_test.rs    # Integration tests for the entire app
Cargo.toml

## Setup and Run

1. Install Rust from [rust-lang.org](https://www.rust-lang.org/).
2. Clone the repository.
3. Run the project with:

```bash
cargo run
```

## Approach and Rationale

- **WebSocket Client**: Connects to Binance's WebSocket to stream OHLC data in real-time.
- **Volatility Calculation**: Utilizes a rolling time window and calculates standard deviation of price changes.
- **Tests**: Ensures WebSocket streams are working and volatility estimates are accurate.
