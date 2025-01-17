# Rust Volatility

## Overview

The `rust-volatility` repository is a Rust-based project designed to calculate and monitor the rolling volatility of cryptocurrency price streams in real-time. The project uses WebSocket (WSS) connections to receive live price data from exchanges and exposes the calculated volatility metrics via a Prometheus endpoint for easy integration with monitoring systems.

---

## Repository Structure

The repository is organized into two primary sub-directories:

1. **`binance_volatility/`**
   - Contains the implementation for monitoring cryptocurrency price streams from the Binance exchange.
   - **Features**:
     - Establishes WebSocket connections to Binance's public API to fetch price updates for specified trading pairs.
     - Uses an unbounded channel (`mpsc::UnboundedChannel`) to decouple message fetching (from Binance) and processing (volatility calculations).
     - Prometheus integration for exposing calculated volatility metrics.
   - **Key Files**:
     - `src/client/mod.rs`: Handles WebSocket connections and message processing for Binance.
     - `src/math/mod.rs`: Implements the rolling window volatility calculation logic.
     - `src/main.rs`: Orchestrates tasks for WebSocket streaming, volatility calculation, and Prometheus metrics server.
 

2. **`uniswap_volatility/`**
   - Contains the implementation for monitoring cryptocurrency price streams from Uniswap.
   - **Features**:
     - Uses the Ethereum blockchain to fetch Uniswap price updates via Infura or another Ethereum node provider.
     - Employs a similar architecture with unbounded channels to decouple data fetching and processing tasks.
     - Prometheus integration for metrics exposure.

   - **Key Files**:
     - `src/client/mod.rs`: Implements the Ethereum client connection logic.
     - `src/math/mod.rs`: Implements the rolling window volatility calculation logic.
     - `src/main.rs`: Coordinates Ethereum client interactions, volatility calculation, and metrics serving.

---

## Architectural Choices

### 1. WebSocket Connections
Both `binance_volatility` and `uniswap_volatility` use WebSocket connections to fetch real-time price data efficiently:
- **Why WebSockets?**
  - WebSocket is a lightweight, full-duplex communication protocol ideal for real-time data streams.
  - It minimizes latency compared to traditional REST APIs by maintaining a persistent connection.

### 2. Unbounded Channels
Unbounded channels (`mpsc::UnboundedChannel`) are used to decouple data fetching from data processing:
- **Design Benefits**:
  - **Asynchronous Decoupling**: The WebSocket connection and price message processing run in separate tasks, ensuring non-blocking behavior.
  - **Scalability**: Multiple data producers (e.g., WebSocket streams) can feed into a single processing pipeline.
  - **Error Isolation**: Issues in one part of the system (e.g., WebSocket reconnection) do not block or interrupt the other tasks.

### 3. Prometheus Integration
Calculated volatility metrics are exposed through a Prometheus-compatible HTTP endpoint:
- Each trading pair is represented by a labeled Prometheus gauge.
- This enables seamless integration with monitoring tools like Grafana for visualization and alerting.

---

## Implementation Highlights

### 1. Client Connections
- **Binance Client**:
  - Connects to Binance's WebSocket API (`wss://stream.binance.com:9443/ws`).
  - Streams live price updates (`@kline_1m`) for specified trading pairs.
  - Automatically reconnects in case of connection drops, with exponential backoff.

- **Uniswap Client**:
  - Fetches price updates directly from the Ethereum blockchain.
  - Uses an Ethereum node provider (e.g., Infura) for decentralized access to Uniswap's on-chain price data.

### 2. Rolling Volatility Calculation
- A `VolatilityCalculator` struct implements a rolling-window volatility algorithm:
  - Maintains a fixed-size buffer of recent prices.
  - Calculates standard deviation (volatility) over the buffer.

### 3. Task Coordination
- **Tokio Framework**:
  - Utilized for asynchronous task execution.
  - Tasks include WebSocket data streaming, volatility calculation, and metrics serving.
- **Graceful Shutdown**:
  - A signal handler (`Ctrl+C`) ensures proper cleanup of resources and task termination.

---

## Example Usage

### Binance Volatility
Run the Binance implementation with:
```bash
cargo run --bin binance_volatility
```

### Uniswap Volatility
```bash
cargo run --bin uniswap_volatility
```

### Prometheus Metrics

After starting the application, access the Prometheus metrics endpoints at:
```bash
binance: http://localhost:8080/metrics
uniswap: http://localhost:8081/metrics
```

Queries for Grafana
```bash
binance_volatility{symbol="ethusdc"}
```
```bash
uniswap_volatility{symbol="ethusdc"}
```


## Future Improvements
### 1.	Error Handling Enhancements:
- Introduce bounded channels to prevent unbounded memory usage.
- Add retries and circuit breakers for API and WebSocket connections.

### 2. Dynamic Pair Management:
- Allow runtime addition/removal of trading pairs without restarting the service.

### 3. Performance Optimization:
- Use batching for volatility calculations.
- Reduce locking overhead in shared data structures.

### 4. Various Volatilities:
- Testing different volatilities schemes like the Corwin and Schultz (Parkison HL Vol) for OHLC data is critical for market-making applications.