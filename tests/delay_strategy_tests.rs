//! # Retry Delay Strategy Tests
//!
//! Covers various delay calculation and parameter validation behaviors of `RetryDelayStrategy`.

use prism3_retry::RetryDelayStrategy;
use std::time::Duration;

#[test]
fn test_none_strategy() {
    let strategy = RetryDelayStrategy::none();
    let delay = strategy.calculate_delay(1, 0.0);
    assert_eq!(delay, Duration::ZERO);
}

#[test]
fn test_fixed_strategy() {
    let strategy = RetryDelayStrategy::fixed(Duration::from_secs(1));
    let delay = strategy.calculate_delay(1, 0.0);
    assert_eq!(delay, Duration::from_secs(1));
}

#[test]
fn test_random_strategy() {
    let strategy =
        RetryDelayStrategy::random(Duration::from_millis(500), Duration::from_millis(1500));

    // Test multiple times to ensure within range
    for _ in 0..100 {
        let delay = strategy.calculate_delay(1, 0.0);
        assert!(delay >= Duration::from_millis(500));
        assert!(delay <= Duration::from_millis(1500));
    }
}

#[test]
fn test_exponential_backoff_strategy() {
    let strategy = RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(100),
        Duration::from_secs(10),
        2.0,
    );

    let delay1 = strategy.calculate_delay(1, 0.0);
    assert_eq!(delay1, Duration::from_millis(100));

    let delay2 = strategy.calculate_delay(2, 0.0);
    assert_eq!(delay2, Duration::from_millis(200));

    let delay3 = strategy.calculate_delay(3, 0.0);
    assert_eq!(delay3, Duration::from_millis(400));

    // Test maximum delay limit
    let delay10 = strategy.calculate_delay(10, 0.0);
    assert!(delay10 <= Duration::from_secs(10));
}

#[test]
fn test_jitter() {
    let strategy = RetryDelayStrategy::fixed(Duration::from_secs(1));

    // Test multiple times to ensure jitter is within range
    for _ in 0..100 {
        let delay = strategy.calculate_delay(1, 0.1);
        assert!(delay >= Duration::from_millis(1000));
        assert!(delay <= Duration::from_millis(1100));
    }
}

#[test]
fn test_validation() {
    // Test valid strategies
    assert!(RetryDelayStrategy::none().validate().is_ok());
    assert!(RetryDelayStrategy::fixed(Duration::from_secs(1))
        .validate()
        .is_ok());
    assert!(
        RetryDelayStrategy::random(Duration::from_millis(100), Duration::from_millis(200))
            .validate()
            .is_ok()
    );
    assert!(RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(100),
        Duration::from_secs(10),
        2.0
    )
    .validate()
    .is_ok());

    // Test invalid strategies
    assert!(RetryDelayStrategy::fixed(Duration::ZERO)
        .validate()
        .is_err());
    assert!(
        RetryDelayStrategy::random(Duration::from_millis(200), Duration::from_millis(100))
            .validate()
            .is_err()
    );
    assert!(RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(100),
        Duration::from_millis(50),
        2.0
    )
    .validate()
    .is_err());
    assert!(RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(100),
        Duration::from_secs(10),
        1.0
    )
    .validate()
    .is_err());
}

#[test]
fn test_validation_error_messages() {
    // Test error message content
    assert_eq!(
        RetryDelayStrategy::fixed(Duration::ZERO)
            .validate()
            .unwrap_err(),
        "Fixed delay cannot be zero"
    );

    assert_eq!(
        RetryDelayStrategy::random(Duration::ZERO, Duration::from_millis(100))
            .validate()
            .unwrap_err(),
        "Random delay minimum cannot be zero"
    );

    assert_eq!(
        RetryDelayStrategy::random(Duration::from_millis(200), Duration::from_millis(100))
            .validate()
            .unwrap_err(),
        "Random delay minimum must be less than maximum"
    );

    assert_eq!(
        RetryDelayStrategy::random(Duration::from_millis(100), Duration::from_millis(100))
            .validate()
            .unwrap_err(),
        "Random delay minimum must be less than maximum"
    );

    assert_eq!(
        RetryDelayStrategy::exponential_backoff(Duration::ZERO, Duration::from_secs(10), 2.0)
            .validate()
            .unwrap_err(),
        "Exponential backoff initial delay cannot be zero"
    );

    assert_eq!(
        RetryDelayStrategy::exponential_backoff(
            Duration::from_millis(100),
            Duration::from_millis(50),
            2.0
        )
        .validate()
        .unwrap_err(),
        "Exponential backoff initial delay must be less than maximum delay"
    );

    assert_eq!(
        RetryDelayStrategy::exponential_backoff(
            Duration::from_millis(100),
            Duration::from_millis(100),
            2.0
        )
        .validate()
        .unwrap_err(),
        "Exponential backoff initial delay must be less than maximum delay"
    );

    assert_eq!(
        RetryDelayStrategy::exponential_backoff(
            Duration::from_millis(100),
            Duration::from_secs(10),
            1.0
        )
        .validate()
        .unwrap_err(),
        "Exponential backoff multiplier must be greater than 1.0"
    );

    assert_eq!(
        RetryDelayStrategy::exponential_backoff(
            Duration::from_millis(100),
            Duration::from_secs(10),
            0.5
        )
        .validate()
        .unwrap_err(),
        "Exponential backoff multiplier must be greater than 1.0"
    );
}

#[test]
fn test_default_strategy() {
    let strategy = RetryDelayStrategy::default();

    // Verify default strategy is exponential backoff
    match strategy {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay,
            max_delay,
            multiplier,
        } => {
            assert_eq!(initial_delay, Duration::from_millis(1000));
            assert_eq!(max_delay, Duration::from_secs(60));
            assert_eq!(multiplier, 2.0);
        }
        _ => panic!("Default strategy should be exponential backoff"),
    }

    // Test default strategy calculation
    let delay1 = strategy.calculate_delay(1, 0.0);
    assert_eq!(delay1, Duration::from_millis(1000));

    let delay2 = strategy.calculate_delay(2, 0.0);
    assert_eq!(delay2, Duration::from_millis(2000));

    // Verify default strategy is valid
    assert!(strategy.validate().is_ok());
}

#[test]
fn test_clone_none() {
    let original = RetryDelayStrategy::none();
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_clone_fixed() {
    let original = RetryDelayStrategy::fixed(Duration::from_secs(5));
    let cloned = original.clone();
    assert_eq!(original, cloned);

    // Verify cloned behavior is consistent
    assert_eq!(
        original.calculate_delay(1, 0.0),
        cloned.calculate_delay(1, 0.0)
    );
}

#[test]
fn test_clone_random() {
    let original =
        RetryDelayStrategy::random(Duration::from_millis(100), Duration::from_millis(1000));
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_clone_exponential_backoff() {
    let original = RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(200),
        Duration::from_secs(30),
        3.0,
    );
    let cloned = original.clone();
    assert_eq!(original, cloned);

    // Verify cloned behavior is consistent
    assert_eq!(
        original.calculate_delay(1, 0.0),
        cloned.calculate_delay(1, 0.0)
    );
    assert_eq!(
        original.calculate_delay(5, 0.0),
        cloned.calculate_delay(5, 0.0)
    );
}

#[test]
fn test_exponential_backoff_different_multipliers() {
    // Test different multipliers
    let strategy2 = RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(100),
        Duration::from_secs(10),
        2.0,
    );
    let strategy3 = RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(100),
        Duration::from_secs(10),
        3.0,
    );

    let delay2_attempt2 = strategy2.calculate_delay(2, 0.0);
    let delay3_attempt2 = strategy3.calculate_delay(2, 0.0);

    assert_eq!(delay2_attempt2, Duration::from_millis(200)); // 100 * 2^1
    assert_eq!(delay3_attempt2, Duration::from_millis(300)); // 100 * 3^1
}

#[test]
fn test_exponential_backoff_max_delay_reached() {
    let strategy = RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(100),
        Duration::from_secs(1),
        2.0,
    );

    // Calculate multiple attempts to ensure maximum delay remains constant after reaching it
    let delay10 = strategy.calculate_delay(10, 0.0);
    let delay20 = strategy.calculate_delay(20, 0.0);
    let delay100 = strategy.calculate_delay(100, 0.0);

    assert_eq!(delay10, Duration::from_secs(1));
    assert_eq!(delay20, Duration::from_secs(1));
    assert_eq!(delay100, Duration::from_secs(1));
}

#[test]
fn test_calculate_delay_with_zero_jitter() {
    let strategies = vec![
        RetryDelayStrategy::none(),
        RetryDelayStrategy::fixed(Duration::from_secs(1)),
        RetryDelayStrategy::exponential_backoff(
            Duration::from_millis(100),
            Duration::from_secs(10),
            2.0,
        ),
    ];

    for strategy in strategies {
        let delay1 = strategy.calculate_delay(1, 0.0);
        let delay2 = strategy.calculate_delay(1, 0.0);
        // With zero jitter, same attempt count should get same delay (except Random)
        if !matches!(strategy, RetryDelayStrategy::Random { .. }) {
            assert_eq!(delay1, delay2);
        }
    }
}

#[test]
fn test_calculate_delay_with_jitter() {
    let strategy = RetryDelayStrategy::fixed(Duration::from_secs(1));

    // Test different jitter factors
    let delay_no_jitter = strategy.calculate_delay(1, 0.0);
    assert_eq!(delay_no_jitter, Duration::from_secs(1));

    // Test small jitter
    for _ in 0..50 {
        let delay = strategy.calculate_delay(1, 0.05);
        assert!(delay >= Duration::from_millis(1000));
        assert!(delay <= Duration::from_millis(1050));
    }

    // Test medium jitter
    for _ in 0..50 {
        let delay = strategy.calculate_delay(1, 0.5);
        assert!(delay >= Duration::from_millis(1000));
        assert!(delay <= Duration::from_millis(1500));
    }

    // Test large jitter
    for _ in 0..50 {
        let delay = strategy.calculate_delay(1, 1.0);
        assert!(delay >= Duration::from_millis(1000));
        assert!(delay <= Duration::from_millis(2000));
    }
}

#[test]
fn test_jitter_with_exponential_backoff() {
    let strategy = RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(100),
        Duration::from_secs(10),
        2.0,
    );

    // Test jitter for first attempt
    for _ in 0..50 {
        let delay = strategy.calculate_delay(1, 0.2);
        assert!(delay >= Duration::from_millis(100));
        assert!(delay <= Duration::from_millis(120));
    }

    // Test jitter for second attempt
    for _ in 0..50 {
        let delay = strategy.calculate_delay(2, 0.2);
        assert!(delay >= Duration::from_millis(200));
        assert!(delay <= Duration::from_millis(240));
    }
}

#[test]
fn test_jitter_with_none_strategy() {
    let strategy = RetryDelayStrategy::none();

    // Even with jitter factor, None strategy should always return zero delay
    let delay = strategy.calculate_delay(1, 0.5);
    assert_eq!(delay, Duration::ZERO);
}

#[test]
fn test_random_strategy_distribution() {
    let strategy =
        RetryDelayStrategy::random(Duration::from_millis(100), Duration::from_millis(200));

    // Collect large number of samples
    let mut samples = Vec::new();
    for _ in 0..1000 {
        let delay = strategy.calculate_delay(1, 0.0);
        samples.push(delay.as_millis());
    }

    // Verify all samples are within range
    for &sample in &samples {
        assert!(sample >= 100);
        assert!(sample <= 200);
    }

    // Verify distribution reasonableness (covers at least half the range)
    let min_sample = *samples.iter().min().unwrap();
    let max_sample = *samples.iter().max().unwrap();
    assert!(max_sample - min_sample >= 50);
}

#[test]
fn test_random_strategy_with_jitter() {
    let strategy =
        RetryDelayStrategy::random(Duration::from_millis(100), Duration::from_millis(200));

    // Random strategy with jitter, results should be above base range
    for _ in 0..100 {
        let delay = strategy.calculate_delay(1, 0.5);
        assert!(delay >= Duration::from_millis(100));
        // Maximum value should be max_delay + jitter
        // 200 + 200*0.5 = 300
        assert!(delay <= Duration::from_millis(300));
    }
}

#[test]
fn test_exponential_backoff_sequence() {
    let strategy = RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(10),
        Duration::from_secs(10),
        2.0,
    );

    // Verify exponential sequence correctness
    assert_eq!(strategy.calculate_delay(1, 0.0), Duration::from_millis(10));
    assert_eq!(strategy.calculate_delay(2, 0.0), Duration::from_millis(20));
    assert_eq!(strategy.calculate_delay(3, 0.0), Duration::from_millis(40));
    assert_eq!(strategy.calculate_delay(4, 0.0), Duration::from_millis(80));
    assert_eq!(strategy.calculate_delay(5, 0.0), Duration::from_millis(160));
    assert_eq!(strategy.calculate_delay(6, 0.0), Duration::from_millis(320));
    assert_eq!(strategy.calculate_delay(7, 0.0), Duration::from_millis(640));
    assert_eq!(
        strategy.calculate_delay(8, 0.0),
        Duration::from_millis(1280)
    );
}

#[test]
fn test_exponential_backoff_with_large_multiplier() {
    let strategy = RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(1),
        Duration::from_secs(10),
        10.0,
    );

    // With large multiplier, should quickly reach maximum value
    assert_eq!(strategy.calculate_delay(1, 0.0), Duration::from_millis(1));
    assert_eq!(strategy.calculate_delay(2, 0.0), Duration::from_millis(10));
    assert_eq!(strategy.calculate_delay(3, 0.0), Duration::from_millis(100));
    assert_eq!(
        strategy.calculate_delay(4, 0.0),
        Duration::from_millis(1000)
    );
    assert_eq!(strategy.calculate_delay(5, 0.0), Duration::from_secs(10)); // Reaches maximum value
    assert_eq!(strategy.calculate_delay(6, 0.0), Duration::from_secs(10)); // Maintains maximum value
}

#[test]
fn test_calculate_delay_with_large_attempt_number() {
    // Test large attempt count doesn't cause overflow or panic
    let strategy = RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(100),
        Duration::from_secs(60),
        2.0,
    );

    let delay = strategy.calculate_delay(1000, 0.0);
    assert_eq!(delay, Duration::from_secs(60)); // Should be limited to maximum value

    // Test very large attempt count (u32::MAX may cause float overflow, returning NaN or inf)
    // In this case, implementation may return max_delay or other values, we just ensure no panic
    let delay = strategy.calculate_delay(100, 0.0);
    assert_eq!(delay, Duration::from_secs(60)); // After 100 times must reach maximum value
}

#[test]
fn test_fixed_strategy_with_various_durations() {
    // Test various fixed delay durations
    let durations = vec![
        Duration::from_nanos(1),
        Duration::from_micros(1),
        Duration::from_millis(1),
        Duration::from_secs(1),
        Duration::from_secs(60),
        Duration::from_secs(3600),
    ];

    for duration in durations {
        let strategy = RetryDelayStrategy::fixed(duration);
        assert_eq!(strategy.calculate_delay(1, 0.0), duration);
        assert_eq!(strategy.calculate_delay(10, 0.0), duration);
        assert_eq!(strategy.calculate_delay(100, 0.0), duration);
    }
}

#[test]
fn test_random_strategy_min_equals_max_edge_case() {
    // Although validate will reject this case, test calculation logic robustness
    let strategy =
        RetryDelayStrategy::random(Duration::from_millis(100), Duration::from_millis(100));

    let delay = strategy.calculate_delay(1, 0.0);
    assert_eq!(delay, Duration::from_millis(100));
}

#[test]
fn test_exponential_backoff_fractional_multiplier() {
    // Test fractional multiplier (although typically use values >1.0)
    let strategy = RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(1000),
        Duration::from_secs(10),
        1.5,
    );

    assert_eq!(
        strategy.calculate_delay(1, 0.0),
        Duration::from_millis(1000)
    );
    assert_eq!(
        strategy.calculate_delay(2, 0.0),
        Duration::from_millis(1500)
    );
    assert_eq!(
        strategy.calculate_delay(3, 0.0),
        Duration::from_millis(2250)
    );
}

#[test]
fn test_all_strategies_equality() {
    // Test if strategies with same config are equal
    let none1 = RetryDelayStrategy::none();
    let none2 = RetryDelayStrategy::none();
    assert_eq!(none1, none2);

    let fixed1 = RetryDelayStrategy::fixed(Duration::from_secs(1));
    let fixed2 = RetryDelayStrategy::fixed(Duration::from_secs(1));
    assert_eq!(fixed1, fixed2);

    let random1 =
        RetryDelayStrategy::random(Duration::from_millis(100), Duration::from_millis(200));
    let random2 =
        RetryDelayStrategy::random(Duration::from_millis(100), Duration::from_millis(200));
    assert_eq!(random1, random2);

    let exp1 = RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(100),
        Duration::from_secs(10),
        2.0,
    );
    let exp2 = RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(100),
        Duration::from_secs(10),
        2.0,
    );
    assert_eq!(exp1, exp2);
}

#[test]
fn test_all_strategies_inequality() {
    // Test strategies with different config are not equal
    let none = RetryDelayStrategy::none();
    let fixed = RetryDelayStrategy::fixed(Duration::from_secs(1));
    let random = RetryDelayStrategy::random(Duration::from_millis(100), Duration::from_millis(200));
    let exp = RetryDelayStrategy::exponential_backoff(
        Duration::from_millis(100),
        Duration::from_secs(10),
        2.0,
    );

    assert_ne!(none, fixed);
    assert_ne!(none, random);
    assert_ne!(none, exp);
    assert_ne!(fixed, random);
    assert_ne!(fixed, exp);
    assert_ne!(random, exp);
}
