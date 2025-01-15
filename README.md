**RUST pet project**

Volatility estimator in `Rust` for the ETH/USDC asset pair. 

The goal is to listen to two different data sources:

--One source is on-chain (e.g. decentralized exchange data on ethereum mainnet)

--The second source is off-chain (e.g. data from a centralized exchange)

Using / combining data from your sources, it continuously estimates volatility over a rolling time window.
