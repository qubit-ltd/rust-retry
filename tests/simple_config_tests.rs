/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # SimpleRetryConfig Integration Tests
//!
//! Tests public API functionality of SimpleRetryConfig.

use qubit_retry::{RetryConfig, RetryDelayStrategy, SimpleRetryConfig};
use std::time::Duration;

// === Basic config tests ===

#[test]
fn test_default_simple_retry_config() {
    let config = SimpleRetryConfig::new();

    assert_eq!(config.max_attempts(), 5);
    assert_eq!(config.max_duration(), None);
    assert_eq!(config.jitter_factor(), 0.0);
    assert_eq!(config.operation_timeout(), None);
}

#[test]
fn test_simple_retry_config_with_params() {
    let config = SimpleRetryConfig::with_params(
        10,
        RetryDelayStrategy::fixed(Duration::from_secs(2)),
        0.2,
        Some(Duration::from_secs(60)),
        Some(Duration::from_secs(5)),
    );

    assert_eq!(config.max_attempts(), 10);
    assert_eq!(config.max_duration(), Some(Duration::from_secs(60)));
    assert_eq!(config.jitter_factor(), 0.2);
    assert_eq!(config.operation_timeout(), Some(Duration::from_secs(5)));

    match config.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => assert_eq!(delay, Duration::from_secs(2)),
        _ => panic!("Expected Fixed strategy"),
    }
}

#[test]
fn test_set_max_attempts() {
    let mut config = SimpleRetryConfig::new();
    config.set_max_attempts(10);
    assert_eq!(config.max_attempts(), 10);
}

#[test]
fn test_set_max_attempts_multiple_times() {
    let mut config = SimpleRetryConfig::new();
    config.set_max_attempts(3);
    assert_eq!(config.max_attempts(), 3);

    config.set_max_attempts(7);
    assert_eq!(config.max_attempts(), 7);

    config.set_max_attempts(15);
    assert_eq!(config.max_attempts(), 15);
}

#[test]
fn test_set_max_duration() {
    let mut config = SimpleRetryConfig::new();
    let duration = Duration::from_secs(30);
    config.set_max_duration(Some(duration));
    assert_eq!(config.max_duration(), Some(duration));

    config.set_max_duration(None);
    assert_eq!(config.max_duration(), None);
}

#[test]
fn test_set_max_duration_multiple_times() {
    let mut config = SimpleRetryConfig::new();

    config.set_max_duration(Some(Duration::from_secs(10)));
    assert_eq!(config.max_duration(), Some(Duration::from_secs(10)));

    config.set_max_duration(Some(Duration::from_secs(30)));
    assert_eq!(config.max_duration(), Some(Duration::from_secs(30)));

    config.set_max_duration(None);
    assert_eq!(config.max_duration(), None);
}

#[test]
fn test_set_operation_timeout() {
    let mut config = SimpleRetryConfig::new();

    config.set_operation_timeout(Some(Duration::from_secs(5)));
    assert_eq!(config.operation_timeout(), Some(Duration::from_secs(5)));

    config.set_operation_timeout(None);
    assert_eq!(config.operation_timeout(), None);
}

#[test]
fn test_set_operation_timeout_multiple_times() {
    let mut config = SimpleRetryConfig::new();

    config.set_operation_timeout(Some(Duration::from_secs(3)));
    assert_eq!(config.operation_timeout(), Some(Duration::from_secs(3)));

    config.set_operation_timeout(Some(Duration::from_secs(10)));
    assert_eq!(config.operation_timeout(), Some(Duration::from_secs(10)));

    config.set_operation_timeout(None);
    assert_eq!(config.operation_timeout(), None);
}

#[test]
fn test_set_jitter_factor() {
    let mut config = SimpleRetryConfig::new();
    config.set_jitter_factor(0.2);
    assert_eq!(config.jitter_factor(), 0.2);
}

#[test]
fn test_set_jitter_factor_multiple_times() {
    let mut config = SimpleRetryConfig::new();

    config.set_jitter_factor(0.1);
    assert_eq!(config.jitter_factor(), 0.1);

    config.set_jitter_factor(0.5);
    assert_eq!(config.jitter_factor(), 0.5);

    config.set_jitter_factor(1.0);
    assert_eq!(config.jitter_factor(), 1.0);
}

#[test]
fn test_set_unlimited_duration() {
    let mut config = SimpleRetryConfig::new();
    config.set_max_duration(Some(Duration::from_secs(60)));
    assert_eq!(config.max_duration(), Some(Duration::from_secs(60)));

    config.set_unlimited_duration();
    assert_eq!(config.max_duration(), None);
}

#[test]
fn test_set_unlimited_operation_timeout() {
    let mut config = SimpleRetryConfig::new();
    config.set_operation_timeout(Some(Duration::from_secs(5)));
    assert_eq!(config.operation_timeout(), Some(Duration::from_secs(5)));

    config.set_unlimited_operation_timeout();
    assert_eq!(config.operation_timeout(), None);
}

// === delaystrategytest ===

#[test]
fn test_set_delay_strategies() {
    let mut config = SimpleRetryConfig::new();

    // Test fixed delay
    config.set_fixed_delay_strategy(Duration::from_secs(2));
    match config.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => assert_eq!(delay, Duration::from_secs(2)),
        _ => panic!("Expected Fixed strategy"),
    }

    // Test random delay
    config.set_random_delay_strategy(Duration::from_millis(500), Duration::from_millis(1500));
    match config.delay_strategy() {
        RetryDelayStrategy::Random {
            min_delay,
            max_delay,
        } => {
            assert_eq!(min_delay, Duration::from_millis(500));
            assert_eq!(max_delay, Duration::from_millis(1500));
        }
        _ => panic!("Expected Random strategy"),
    }

    // Test exponential backoff
    config.set_exponential_backoff_strategy(
        Duration::from_millis(100),
        Duration::from_secs(10),
        2.5,
    );
    match config.delay_strategy() {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay,
            max_delay,
            multiplier,
        } => {
            assert_eq!(initial_delay, Duration::from_millis(100));
            assert_eq!(max_delay, Duration::from_secs(10));
            assert_eq!(multiplier, 2.5);
        }
        _ => panic!("Expected ExponentialBackoff strategy"),
    }

    // Test no delay
    config.set_no_delay_strategy();
    match config.delay_strategy() {
        RetryDelayStrategy::None => {}
        _ => panic!("Expected None strategy"),
    }
}

#[test]
fn test_set_fixed_delay_strategy() {
    let mut config = SimpleRetryConfig::new();
    config.set_fixed_delay_strategy(Duration::from_secs(3));

    match config.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => {
            assert_eq!(delay, Duration::from_secs(3));
        }
        _ => panic!("Expected Fixed strategy"),
    }
}

#[test]
fn test_set_fixed_delay_strategy_multiple_times() {
    let mut config = SimpleRetryConfig::new();

    config.set_fixed_delay_strategy(Duration::from_secs(1));
    match config.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => assert_eq!(delay, Duration::from_secs(1)),
        _ => panic!("Expected Fixed strategy"),
    }

    config.set_fixed_delay_strategy(Duration::from_secs(5));
    match config.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => assert_eq!(delay, Duration::from_secs(5)),
        _ => panic!("Expected Fixed strategy"),
    }
}

#[test]
fn test_set_random_delay_strategy() {
    let mut config = SimpleRetryConfig::new();
    config.set_random_delay_strategy(Duration::from_millis(100), Duration::from_millis(2000));

    match config.delay_strategy() {
        RetryDelayStrategy::Random {
            min_delay,
            max_delay,
        } => {
            assert_eq!(min_delay, Duration::from_millis(100));
            assert_eq!(max_delay, Duration::from_millis(2000));
        }
        _ => panic!("Expected Random strategy"),
    }
}

#[test]
fn test_set_random_delay_strategy_multiple_times() {
    let mut config = SimpleRetryConfig::new();

    config.set_random_delay_strategy(Duration::from_millis(100), Duration::from_millis(1000));
    match config.delay_strategy() {
        RetryDelayStrategy::Random {
            min_delay,
            max_delay,
        } => {
            assert_eq!(min_delay, Duration::from_millis(100));
            assert_eq!(max_delay, Duration::from_millis(1000));
        }
        _ => panic!("Expected Random strategy"),
    }

    config.set_random_delay_strategy(Duration::from_millis(500), Duration::from_millis(5000));
    match config.delay_strategy() {
        RetryDelayStrategy::Random {
            min_delay,
            max_delay,
        } => {
            assert_eq!(min_delay, Duration::from_millis(500));
            assert_eq!(max_delay, Duration::from_millis(5000));
        }
        _ => panic!("Expected Random strategy"),
    }
}

#[test]
fn test_set_exponential_backoff_strategy() {
    let mut config = SimpleRetryConfig::new();
    config.set_exponential_backoff_strategy(
        Duration::from_millis(200),
        Duration::from_secs(20),
        3.0,
    );

    match config.delay_strategy() {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay,
            max_delay,
            multiplier,
        } => {
            assert_eq!(initial_delay, Duration::from_millis(200));
            assert_eq!(max_delay, Duration::from_secs(20));
            assert_eq!(multiplier, 3.0);
        }
        _ => panic!("Expected ExponentialBackoff strategy"),
    }
}

#[test]
fn test_set_exponential_backoff_strategy_multiple_times() {
    let mut config = SimpleRetryConfig::new();

    config.set_exponential_backoff_strategy(
        Duration::from_millis(100),
        Duration::from_secs(10),
        2.0,
    );
    match config.delay_strategy() {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay,
            max_delay,
            multiplier,
        } => {
            assert_eq!(initial_delay, Duration::from_millis(100));
            assert_eq!(max_delay, Duration::from_secs(10));
            assert_eq!(multiplier, 2.0);
        }
        _ => panic!("Expected ExponentialBackoff strategy"),
    }

    config.set_exponential_backoff_strategy(
        Duration::from_millis(500),
        Duration::from_secs(60),
        3.5,
    );
    match config.delay_strategy() {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay,
            max_delay,
            multiplier,
        } => {
            assert_eq!(initial_delay, Duration::from_millis(500));
            assert_eq!(max_delay, Duration::from_secs(60));
            assert_eq!(multiplier, 3.5);
        }
        _ => panic!("Expected ExponentialBackoff strategy"),
    }
}

#[test]
fn test_set_no_delay_strategy() {
    let mut config = SimpleRetryConfig::new();
    config.set_no_delay_strategy();

    match config.delay_strategy() {
        RetryDelayStrategy::None => {}
        _ => panic!("Expected None strategy"),
    }
}

#[test]
fn test_delay_strategy_switching() {
    let mut config = SimpleRetryConfig::new();

    // Switch from default strategy to fixed delay
    config.set_fixed_delay_strategy(Duration::from_secs(1));
    match config.delay_strategy() {
        RetryDelayStrategy::Fixed { .. } => {}
        _ => panic!("Expected Fixed strategy"),
    }

    // Switch to random delay
    config.set_random_delay_strategy(Duration::from_millis(100), Duration::from_millis(1000));
    match config.delay_strategy() {
        RetryDelayStrategy::Random { .. } => {}
        _ => panic!("Expected Random strategy"),
    }

    // Switch to exponential backoff
    config.set_exponential_backoff_strategy(
        Duration::from_millis(100),
        Duration::from_secs(10),
        2.0,
    );
    match config.delay_strategy() {
        RetryDelayStrategy::ExponentialBackoff { .. } => {}
        _ => panic!("Expected ExponentialBackoff strategy"),
    }

    // Switch to no delay
    config.set_no_delay_strategy();
    match config.delay_strategy() {
        RetryDelayStrategy::None => {}
        _ => panic!("Expected None strategy"),
    }
}

// === Method chaining tests ===

#[test]
fn test_fluent_api() {
    let mut config = SimpleRetryConfig::new();

    config
        .set_max_attempts(8)
        .set_jitter_factor(0.15)
        .set_max_duration(Some(Duration::from_secs(120)))
        .set_operation_timeout(Some(Duration::from_secs(10)))
        .set_fixed_delay_strategy(Duration::from_secs(5));

    assert_eq!(config.max_attempts(), 8);
    assert_eq!(config.jitter_factor(), 0.15);
    assert_eq!(config.max_duration(), Some(Duration::from_secs(120)));
    assert_eq!(config.operation_timeout(), Some(Duration::from_secs(10)));

    match config.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => {
            assert_eq!(delay, Duration::from_secs(5));
        }
        _ => panic!("Expected Fixed strategy"),
    }
}

#[test]
fn test_fluent_api_complex_chain() {
    let mut config = SimpleRetryConfig::new();

    config
        .set_max_attempts(10)
        .set_exponential_backoff_strategy(Duration::from_millis(500), Duration::from_secs(30), 2.5)
        .set_jitter_factor(0.2)
        .set_max_duration(Some(Duration::from_secs(300)))
        .set_operation_timeout(Some(Duration::from_secs(15)));

    assert_eq!(config.max_attempts(), 10);
    assert_eq!(config.jitter_factor(), 0.2);
    assert_eq!(config.max_duration(), Some(Duration::from_secs(300)));
    assert_eq!(config.operation_timeout(), Some(Duration::from_secs(15)));

    match config.delay_strategy() {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay,
            max_delay,
            multiplier,
        } => {
            assert_eq!(initial_delay, Duration::from_millis(500));
            assert_eq!(max_delay, Duration::from_secs(30));
            assert_eq!(multiplier, 2.5);
        }
        _ => panic!("Expected ExponentialBackoff strategy"),
    }
}

#[test]
fn test_fluent_api_with_unlimited() {
    let mut config = SimpleRetryConfig::new();

    config
        .set_max_attempts(5)
        .set_max_duration(Some(Duration::from_secs(60)))
        .set_operation_timeout(Some(Duration::from_secs(5)))
        .set_unlimited_duration()
        .set_unlimited_operation_timeout();

    assert_eq!(config.max_attempts(), 5);
    assert_eq!(config.max_duration(), None);
    assert_eq!(config.operation_timeout(), None);
}

// === Default trait test ===

#[test]
fn test_default_trait() {
    let config = SimpleRetryConfig::default();

    assert_eq!(config.max_attempts(), 5);
    assert_eq!(config.max_duration(), None);
    assert_eq!(config.jitter_factor(), 0.0);
    assert_eq!(config.operation_timeout(), None);
}

#[test]
fn test_default_vs_new() {
    let config1 = SimpleRetryConfig::default();
    let config2 = SimpleRetryConfig::new();

    assert_eq!(config1.max_attempts(), config2.max_attempts());
    assert_eq!(config1.max_duration(), config2.max_duration());
    assert_eq!(config1.jitter_factor(), config2.jitter_factor());
    assert_eq!(config1.operation_timeout(), config2.operation_timeout());
}

// === Boundary condition tests ===

#[test]
fn test_max_attempts_boundary() {
    let mut config = SimpleRetryConfig::new();

    // Test minimum value
    config.set_max_attempts(1);
    assert_eq!(config.max_attempts(), 1);

    // Test zero value (not recommended, but should be storable)
    config.set_max_attempts(0);
    assert_eq!(config.max_attempts(), 0);

    // Test large value
    config.set_max_attempts(1000);
    assert_eq!(config.max_attempts(), 1000);

    // Test very large value
    config.set_max_attempts(u32::MAX);
    assert_eq!(config.max_attempts(), u32::MAX);
}

#[test]
fn test_jitter_factor_boundary() {
    let mut config = SimpleRetryConfig::new();

    // Test zero value
    config.set_jitter_factor(0.0);
    assert_eq!(config.jitter_factor(), 0.0);

    // Test negative value (not recommended, but should be storable)
    config.set_jitter_factor(-0.5);
    assert_eq!(config.jitter_factor(), -0.5);

    // Test value 1
    config.set_jitter_factor(1.0);
    assert_eq!(config.jitter_factor(), 1.0);

    // Test value greater than 1
    config.set_jitter_factor(2.5);
    assert_eq!(config.jitter_factor(), 2.5);

    // Test very large value
    config.set_jitter_factor(100.0);
    assert_eq!(config.jitter_factor(), 100.0);

    // Test very small value
    config.set_jitter_factor(0.001);
    assert_eq!(config.jitter_factor(), 0.001);
}

#[test]
fn test_max_duration_zero() {
    let mut config = SimpleRetryConfig::new();

    // Set to 0
    config.set_max_duration(Some(Duration::from_millis(0)));
    assert_eq!(config.max_duration(), Some(Duration::from_millis(0)));
}

#[test]
fn test_max_duration_very_large() {
    let mut config = SimpleRetryConfig::new();

    // Set to very large value
    let large_duration = Duration::from_secs(86400 * 365); // 1 year
    config.set_max_duration(Some(large_duration));
    assert_eq!(config.max_duration(), Some(large_duration));
}

#[test]
fn test_operation_timeout_zero() {
    let mut config = SimpleRetryConfig::new();

    // Set to 0
    config.set_operation_timeout(Some(Duration::from_millis(0)));
    assert_eq!(config.operation_timeout(), Some(Duration::from_millis(0)));
}

#[test]
fn test_operation_timeout_very_large() {
    let mut config = SimpleRetryConfig::new();

    // Set to very large value
    let large_duration = Duration::from_secs(3600 * 24); // 1 day
    config.set_operation_timeout(Some(large_duration));
    assert_eq!(config.operation_timeout(), Some(large_duration));
}

#[test]
fn test_delay_strategy_very_small_durations() {
    let mut config = SimpleRetryConfig::new();

    // Test very small fixed delay
    config.set_fixed_delay_strategy(Duration::from_nanos(1));
    match config.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => assert_eq!(delay, Duration::from_nanos(1)),
        _ => panic!("Expected Fixed strategy"),
    }

    // Test very small random delay
    config.set_random_delay_strategy(Duration::from_nanos(1), Duration::from_nanos(10));
    match config.delay_strategy() {
        RetryDelayStrategy::Random {
            min_delay,
            max_delay,
        } => {
            assert_eq!(min_delay, Duration::from_nanos(1));
            assert_eq!(max_delay, Duration::from_nanos(10));
        }
        _ => panic!("Expected Random strategy"),
    }
}

// === Clone test ===

#[test]
fn test_clone() {
    let mut config = SimpleRetryConfig::new();
    config.set_max_attempts(7);
    config.set_jitter_factor(0.25);
    config.set_max_duration(Some(Duration::from_secs(90)));
    config.set_operation_timeout(Some(Duration::from_secs(8)));
    config.set_fixed_delay_strategy(Duration::from_secs(4));

    let cloned_config = config.clone();

    assert_eq!(cloned_config.max_attempts(), 7);
    assert_eq!(cloned_config.jitter_factor(), 0.25);
    assert_eq!(cloned_config.max_duration(), Some(Duration::from_secs(90)));
    assert_eq!(
        cloned_config.operation_timeout(),
        Some(Duration::from_secs(8))
    );

    match cloned_config.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => {
            assert_eq!(delay, Duration::from_secs(4));
        }
        _ => panic!("Expected Fixed strategy"),
    }
}

#[test]
fn test_clone_independence() {
    let mut config = SimpleRetryConfig::new();
    config.set_max_attempts(5);

    let mut cloned_config = config.clone();
    cloned_config.set_max_attempts(10);

    // Modifying clone should not affect original config
    assert_eq!(config.max_attempts(), 5);
    assert_eq!(cloned_config.max_attempts(), 10);
}

#[test]
fn test_clone_with_complex_strategy() {
    let mut config = SimpleRetryConfig::new();
    config.set_exponential_backoff_strategy(
        Duration::from_millis(250),
        Duration::from_secs(45),
        2.8,
    );

    let cloned_config = config.clone();

    match cloned_config.delay_strategy() {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay,
            max_delay,
            multiplier,
        } => {
            assert_eq!(initial_delay, Duration::from_millis(250));
            assert_eq!(max_delay, Duration::from_secs(45));
            assert_eq!(multiplier, 2.8);
        }
        _ => panic!("Expected ExponentialBackoff strategy"),
    }
}

// === Debug test ===

#[test]
fn test_debug_format() {
    let config = SimpleRetryConfig::new();
    let debug_output = format!("{:?}", config);

    // Check Debug output contains key information
    assert!(debug_output.contains("SimpleRetryConfig"));
    assert!(debug_output.contains("max_attempts"));
}

#[test]
fn test_debug_format_with_custom_values() {
    let mut config = SimpleRetryConfig::new();
    config.set_max_attempts(3);
    config.set_jitter_factor(0.5);

    let debug_output = format!("{:?}", config);
    assert!(debug_output.contains("SimpleRetryConfig"));
}

// === Comprehensive scenario tests ===

#[test]
fn test_complete_configuration_flow() {
    // Create config
    let mut config = SimpleRetryConfig::new();

    // Verify default values
    assert_eq!(config.max_attempts(), 5);
    assert_eq!(config.max_duration(), None);

    // Configure retry parameters
    config
        .set_max_attempts(3)
        .set_max_duration(Some(Duration::from_secs(30)))
        .set_operation_timeout(Some(Duration::from_secs(5)))
        .set_exponential_backoff_strategy(Duration::from_millis(100), Duration::from_secs(10), 2.0)
        .set_jitter_factor(0.1);

    // Verify all configurations
    assert_eq!(config.max_attempts(), 3);
    assert_eq!(config.max_duration(), Some(Duration::from_secs(30)));
    assert_eq!(config.operation_timeout(), Some(Duration::from_secs(5)));
    assert_eq!(config.jitter_factor(), 0.1);

    match config.delay_strategy() {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay,
            max_delay,
            multiplier,
        } => {
            assert_eq!(initial_delay, Duration::from_millis(100));
            assert_eq!(max_delay, Duration::from_secs(10));
            assert_eq!(multiplier, 2.0);
        }
        _ => panic!("Expected ExponentialBackoff strategy"),
    }
}

#[test]
fn test_reconfiguration() {
    let mut config = SimpleRetryConfig::new();

    // First configuration
    config
        .set_max_attempts(5)
        .set_fixed_delay_strategy(Duration::from_secs(1));

    assert_eq!(config.max_attempts(), 5);

    // Reconfigure
    config
        .set_max_attempts(10)
        .set_random_delay_strategy(Duration::from_millis(500), Duration::from_millis(2000));

    assert_eq!(config.max_attempts(), 10);
    match config.delay_strategy() {
        RetryDelayStrategy::Random { .. } => {}
        _ => panic!("Expected Random strategy"),
    }
}

#[test]
fn test_with_params_all_combinations() {
    // Test all None cases
    let config1 = SimpleRetryConfig::with_params(1, RetryDelayStrategy::none(), 0.0, None, None);
    assert_eq!(config1.max_attempts(), 1);
    assert_eq!(config1.max_duration(), None);
    assert_eq!(config1.operation_timeout(), None);

    // Test all Some cases
    let config2 = SimpleRetryConfig::with_params(
        10,
        RetryDelayStrategy::fixed(Duration::from_secs(2)),
        0.5,
        Some(Duration::from_secs(60)),
        Some(Duration::from_secs(5)),
    );
    assert_eq!(config2.max_attempts(), 10);
    assert_eq!(config2.max_duration(), Some(Duration::from_secs(60)));
    assert_eq!(config2.operation_timeout(), Some(Duration::from_secs(5)));
}

// === Stress tests ===

#[test]
fn test_rapid_configuration_changes() {
    let mut config = SimpleRetryConfig::new();

    // Rapidly modify config multiple times
    for i in 1..=100 {
        config.set_max_attempts(i);
        assert_eq!(config.max_attempts(), i);
    }

    // Rapidly switch delay strategies
    for _ in 0..50 {
        config.set_fixed_delay_strategy(Duration::from_secs(1));
        config.set_random_delay_strategy(Duration::from_millis(100), Duration::from_millis(1000));
        config.set_exponential_backoff_strategy(
            Duration::from_millis(100),
            Duration::from_secs(10),
            2.0,
        );
        config.set_no_delay_strategy();
    }
}

#[test]
fn test_multiple_clones() {
    let config = SimpleRetryConfig::new();

    // Create multiple clones
    let clones: Vec<_> = (0..100).map(|_| config.clone()).collect();

    // Verify all clones are identical
    for cloned_config in clones {
        assert_eq!(cloned_config.max_attempts(), config.max_attempts());
        assert_eq!(cloned_config.max_duration(), config.max_duration());
        assert_eq!(cloned_config.jitter_factor(), config.jitter_factor());
    }
}
