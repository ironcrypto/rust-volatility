**Volatility Estimator in RUST**

Volatility estimator in `Rust` for the ETH/USDC asset pair. 

The goal is to listen to two different data sources:

- One source is on-chain (e.g. decentralized exchange data on ethereum mainnet)

- The second source is off-chain (e.g. data from a centralized exchange)

Using data from the sources, it continuously estimates two volatilities over a rolling time window, and exposes them to prometheus endpoint.

*Prometheus Queries*:
- uniswap_volatility{symbol="ethusdc"}
- binance_volatility{symbol="ethusdc"}

*Prometheus Endpoint*: see config file /prometheus.yml