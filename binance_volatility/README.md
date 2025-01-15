
# Binance Volatility Estimator

This Rust project implements a Binance WebSocket client to retrieve OHLC data and calculates rolling window volatility based on a given algorithm.


## Setup and Run

1. Clone the repository.
2. Start Prometheus with the /prometheus.yml file
2. Run the project with:

```bash
cargo build
cargo run
```


## Approach and Rationale

- **WebSocket Client**: Connects to Binance's WebSocket to stream OHLC data in real-time.
- **Volatility Calculation**: Utilizes a rolling time window and calculates standard deviation of price changes.
- **Tests**: Ensures WebSocket streams are working and volatility estimates are accurate.
