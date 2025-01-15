use std::collections::VecDeque;
use std::time::{Duration, SystemTime};

pub struct VolatilityCalculator {
    window: VecDeque<(SystemTime, f64)>, // Stores (timestamp, price)
    max_duration: Duration,              // Maximum window size in time
}

impl VolatilityCalculator {
    /// Create a new volatility calculator with a rolling window duration in seconds.
    pub fn new(max_duration_secs: u64) -> Self {
        VolatilityCalculator {
            window: VecDeque::new(),
            max_duration: Duration::from_secs(max_duration_secs),
        }
    }

    /// Add a new price value to the rolling window.
    pub fn add_value(&mut self, price: f64) {
        let now = SystemTime::now();

        // Add the new price with the current timestamp
        self.window.push_back((now, price));

        // Remove prices outside the rolling window
        while let Some((timestamp, _)) = self.window.front() {
            if let Ok(duration) = now.duration_since(*timestamp) {
                if duration > self.max_duration {
                    self.window.pop_front(); // Remove the oldest value
                } else {
                    break;
                }
            } else {
                warn!("Encountered a timestamp in the future: {:?}", timestamp);
                self.window.pop_front();
            }
        }
    }

    /// Calculate the rolling volatility (standard deviation of prices).
    pub fn calculate_volatility(&self) -> Option<f64> {
        if self.window.len() < 2 {
            return None; // Not enough data points for calculation
        }

        let prices: Vec<f64> = self.window.iter().map(|(_, price)| *price).collect();
        let mean: f64 = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance: f64 = prices.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / prices.len() as f64;

        Some(variance.sqrt()) // Standard deviation as volatility
    }
}

