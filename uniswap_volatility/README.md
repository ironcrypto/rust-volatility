
# Uniswap Volatility Estimator

This Rust project implements real-time volatility estimation for the Uniswap V3 ETH/USDC pool.

## Setup and Run

Ensure you have Rust installed and an Infura API key.

```bash
cargo run
```

## Approach and Rationale
	•	WebSocket Client: Connects to Infura’s API to stream data in real-time.
	•	Volatility Calculation: Utilizes a rolling time window and calculates the standard deviation of price changes.
	•	Tests: Ensures streams are working and volatility estimates are accurate.
