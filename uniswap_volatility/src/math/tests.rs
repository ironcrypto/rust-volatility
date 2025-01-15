
#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};
    use std::collections::VecDeque;
    use crate::math::VolatilityCalculator;


    #[test]
    fn test_rolling_volatility() {
        let mut calc = VolatilityCalculator::new(10);

        calc.add_value(100.0);
        calc.add_value(102.0);
        calc.add_value(98.0);
        calc.add_value(101.0);

        let volatility = calc.calculate_volatility().unwrap();
        assert!(volatility > 0.0); // Ensure a non-zero volatility is calculated
    }

    #[test]
    fn test_empty_volatility() {
        let mut calc = VolatilityCalculator::new(10);

        // No data points
        assert!(calc.calculate_volatility().is_none());
    }

    #[test]
    fn test_window_trimming() {
        let mut calc = VolatilityCalculator::new(1); // 1-second window

        calc.add_value(100.0);
        std::thread::sleep(Duration::from_secs(2)); // Wait to exceed the window
        calc.add_value(102.0);

        let volatility = calc.calculate_volatility().unwrap();
        assert!(volatility == 0.0); // Only one value remains in the window
    }
}