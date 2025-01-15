#[cfg(test)]
mod tests {
    use crate::math::VolatilityCalculator;
    use std::time::Duration;
    use std::thread::sleep;


    #[test]
    fn test_volatility_empty_window() {
        let calculator = VolatilityCalculator::new(5);
        let result = calculator.calculate_volatility();
        assert_eq!(result, None, "Volatility should be None for an empty window.");
    }

    #[test]
    fn test_volatility_single_value() {
        let mut calculator = VolatilityCalculator::new(5);
        calculator.add_value(100.0);
        let result = calculator.calculate_volatility();
        assert_eq!(result, None, "Volatility should be None for a single value.");
    }

    #[test]
    fn test_volatility_basic_calculation() {
        let mut calculator = VolatilityCalculator::new(5);
        calculator.add_value(100.0);
        calculator.add_value(110.0);
        calculator.add_value(120.0);

        let result = calculator.calculate_volatility();
        assert!(result.is_some(), "Volatility should be calculable for multiple values.");
        assert!(result.unwrap() > 0.0, "Volatility should be greater than 0.");
    }


    #[test]
    fn test_rolling_window() {
        let mut calculator = VolatilityCalculator::new(30); // 30-second window

        // Add values at different times
        calculator.add_value(100.0);
        sleep(Duration::from_secs(15));
        calculator.add_value(110.0);
        sleep(Duration::from_secs(20));
        calculator.add_value(120.0);

        // Only the last two values should remain (30-second window)
        assert_eq!(calculator.window.len(), 2);

        // Calculate volatility
        let volatility = calculator.calculate_volatility();
        assert!(volatility.is_some());
        println!("Calculated Volatility: {:?}", volatility);
    }
}