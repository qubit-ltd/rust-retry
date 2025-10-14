//! # DefaultRetryConfig Integration Tests
//!
//! Tests public API functionality of DefaultRetryConfig.

use prism3_config::{Config, Configurable};
use prism3_retry::{DefaultRetryConfig, RetryConfig, RetryDelayStrategy};
use std::time::Duration;

// === Basic config tests ===

#[test]
fn test_default_retry_config() {
    let config = DefaultRetryConfig::new();

    assert_eq!(config.max_attempts(), 5);
    assert_eq!(config.max_duration(), None);
    assert_eq!(config.jitter_factor(), 0.0);
}

#[test]
fn test_set_max_attempts() {
    let mut config = DefaultRetryConfig::new();
    config.set_max_attempts(10);
    assert_eq!(config.max_attempts(), 10);
}

#[test]
fn test_set_max_duration() {
    let mut config = DefaultRetryConfig::new();
    let duration = Duration::from_secs(30);
    config.set_max_duration(Some(duration));
    assert_eq!(config.max_duration(), Some(duration));

    config.set_max_duration(None);
    assert_eq!(config.max_duration(), None);
}

#[test]
fn test_set_jitter_factor() {
    let mut config = DefaultRetryConfig::new();
    config.set_jitter_factor(0.2);
    assert_eq!(config.jitter_factor(), 0.2);
}

#[test]
fn test_set_unlimited_duration() {
    let mut config = DefaultRetryConfig::new();
    config.set_max_duration(Some(Duration::from_secs(60)));
    assert_eq!(config.max_duration(), Some(Duration::from_secs(60)));

    config.set_unlimited_duration();
    assert_eq!(config.max_duration(), None);
}

// === delaystrategytest ===

#[test]
fn test_set_delay_strategies() {
    let mut config = DefaultRetryConfig::new();

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
    let mut config = DefaultRetryConfig::new();
    config.set_fixed_delay_strategy(Duration::from_secs(3));

    match config.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => {
            assert_eq!(delay, Duration::from_secs(3));
        }
        _ => panic!("Expected Fixed strategy"),
    }
}

#[test]
fn test_set_random_delay_strategy() {
    let mut config = DefaultRetryConfig::new();
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
fn test_set_exponential_backoff_strategy() {
    let mut config = DefaultRetryConfig::new();
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
fn test_set_no_delay_strategy() {
    let mut config = DefaultRetryConfig::new();
    config.set_no_delay_strategy();

    match config.delay_strategy() {
        RetryDelayStrategy::None => {}
        _ => panic!("Expected None strategy"),
    }
}

// === Configurable trait test ===

#[test]
fn test_with_config() {
    let mut base_config = Config::new();
    base_config
        .set(DefaultRetryConfig::KEY_MAX_ATTEMPTS, 10u32)
        .unwrap();
    base_config
        .set(DefaultRetryConfig::KEY_JITTER_FACTOR, 0.3)
        .unwrap();

    let retry_config = DefaultRetryConfig::with_config(base_config);

    assert_eq!(retry_config.max_attempts(), 10);
    assert_eq!(retry_config.jitter_factor(), 0.3);
}

#[test]
fn test_config_accessor() {
    let config = DefaultRetryConfig::new();
    let inner_config = config.config();

    assert!(inner_config
        .get::<u32>(DefaultRetryConfig::KEY_MAX_ATTEMPTS)
        .is_err());
}

#[test]
fn test_config_mut_accessor() {
    let mut config = DefaultRetryConfig::new();
    let inner_config = config.config_mut();

    inner_config
        .set(DefaultRetryConfig::KEY_MAX_ATTEMPTS, 15u32)
        .unwrap();

    assert_eq!(config.max_attempts(), 15);
}

#[test]
fn test_set_config() {
    let mut config = DefaultRetryConfig::new();

    let mut new_config = Config::new();
    new_config
        .set(DefaultRetryConfig::KEY_MAX_ATTEMPTS, 20u32)
        .unwrap();
    new_config
        .set(DefaultRetryConfig::KEY_JITTER_FACTOR, 0.5)
        .unwrap();

    config.set_config(new_config);

    assert_eq!(config.max_attempts(), 20);
    assert_eq!(config.jitter_factor(), 0.5);
}

// === Load delay strategy from Config tests ===

#[test]
fn test_load_none_strategy_from_config() {
    let mut base_config = Config::new();
    base_config
        .set(DefaultRetryConfig::KEY_DELAY_STRATEGY, "NONE")
        .unwrap();

    let config = DefaultRetryConfig::with_config(base_config);

    match config.delay_strategy() {
        RetryDelayStrategy::None => {}
        _ => panic!("Expected None strategy"),
    }
}

#[test]
fn test_load_fixed_strategy_from_config() {
    let mut base_config = Config::new();
    base_config
        .set(DefaultRetryConfig::KEY_DELAY_STRATEGY, "FIXED")
        .unwrap();
    base_config
        .set(DefaultRetryConfig::KEY_FIXED_DELAY, 3000u64)
        .unwrap();

    let config = DefaultRetryConfig::with_config(base_config);

    match config.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => {
            assert_eq!(delay, Duration::from_millis(3000));
        }
        _ => panic!("Expected Fixed strategy"),
    }
}

#[test]
fn test_load_random_strategy_from_config() {
    let mut base_config = Config::new();
    base_config
        .set(DefaultRetryConfig::KEY_DELAY_STRATEGY, "RANDOM")
        .unwrap();
    base_config
        .set(DefaultRetryConfig::KEY_RANDOM_MIN_DELAY, 500u64)
        .unwrap();
    base_config
        .set(DefaultRetryConfig::KEY_RANDOM_MAX_DELAY, 5000u64)
        .unwrap();

    let config = DefaultRetryConfig::with_config(base_config);

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
fn test_load_exponential_backoff_strategy_from_config() {
    let mut base_config = Config::new();
    base_config
        .set(
            DefaultRetryConfig::KEY_DELAY_STRATEGY,
            "EXPONENTIAL_BACKOFF",
        )
        .unwrap();
    base_config
        .set(DefaultRetryConfig::KEY_BACKOFF_INITIAL_DELAY, 500u64)
        .unwrap();
    base_config
        .set(DefaultRetryConfig::KEY_BACKOFF_MAX_DELAY, 30000u64)
        .unwrap();
    base_config
        .set(DefaultRetryConfig::KEY_BACKOFF_MULTIPLIER, 2.5)
        .unwrap();

    let config = DefaultRetryConfig::with_config(base_config);

    match config.delay_strategy() {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay,
            max_delay,
            multiplier,
        } => {
            assert_eq!(initial_delay, Duration::from_millis(500));
            assert_eq!(max_delay, Duration::from_millis(30000));
            assert_eq!(multiplier, 2.5);
        }
        _ => panic!("Expected ExponentialBackoff strategy"),
    }
}

#[test]
fn test_load_unknown_strategy_from_config() {
    let mut base_config = Config::new();
    base_config
        .set(DefaultRetryConfig::KEY_DELAY_STRATEGY, "UNKNOWN")
        .unwrap();

    let config = DefaultRetryConfig::with_config(base_config);

    // Should return default strategy
    match config.delay_strategy() {
        RetryDelayStrategy::ExponentialBackoff { .. } => {}
        _ => panic!("Expected default ExponentialBackoff strategy"),
    }
}

// === Save delay strategy to Config tests ===

#[test]
fn test_save_none_strategy_to_config() {
    let mut config = DefaultRetryConfig::new();
    config.set_delay_strategy(RetryDelayStrategy::None);

    let inner_config = config.config();
    let strategy = inner_config
        .get_string(DefaultRetryConfig::KEY_DELAY_STRATEGY)
        .unwrap();
    assert_eq!(strategy, "NONE");
}

#[test]
fn test_save_fixed_strategy_to_config() {
    let mut config = DefaultRetryConfig::new();
    config.set_delay_strategy(RetryDelayStrategy::Fixed {
        delay: Duration::from_millis(2500),
    });

    let inner_config = config.config();
    let strategy = inner_config
        .get_string(DefaultRetryConfig::KEY_DELAY_STRATEGY)
        .unwrap();
    assert_eq!(strategy, "FIXED");
    let delay = inner_config
        .get::<u64>(DefaultRetryConfig::KEY_FIXED_DELAY)
        .unwrap();
    assert_eq!(delay, 2500);
}

#[test]
fn test_save_random_strategy_to_config() {
    let mut config = DefaultRetryConfig::new();
    config.set_delay_strategy(RetryDelayStrategy::Random {
        min_delay: Duration::from_millis(300),
        max_delay: Duration::from_millis(3000),
    });

    let inner_config = config.config();
    let strategy = inner_config
        .get_string(DefaultRetryConfig::KEY_DELAY_STRATEGY)
        .unwrap();
    assert_eq!(strategy, "RANDOM");
    let min_delay = inner_config
        .get::<u64>(DefaultRetryConfig::KEY_RANDOM_MIN_DELAY)
        .unwrap();
    assert_eq!(min_delay, 300);
    let max_delay = inner_config
        .get::<u64>(DefaultRetryConfig::KEY_RANDOM_MAX_DELAY)
        .unwrap();
    assert_eq!(max_delay, 3000);
}

#[test]
fn test_save_exponential_backoff_strategy_to_config() {
    let mut config = DefaultRetryConfig::new();
    config.set_delay_strategy(RetryDelayStrategy::ExponentialBackoff {
        initial_delay: Duration::from_millis(800),
        max_delay: Duration::from_millis(40000),
        multiplier: 3.5,
    });

    let inner_config = config.config();
    let strategy = inner_config
        .get_string(DefaultRetryConfig::KEY_DELAY_STRATEGY)
        .unwrap();
    assert_eq!(strategy, "EXPONENTIAL_BACKOFF");
    let initial_delay = inner_config
        .get::<u64>(DefaultRetryConfig::KEY_BACKOFF_INITIAL_DELAY)
        .unwrap();
    assert_eq!(initial_delay, 800);
    let max_delay = inner_config
        .get::<u64>(DefaultRetryConfig::KEY_BACKOFF_MAX_DELAY)
        .unwrap();
    assert_eq!(max_delay, 40000);
    let multiplier = inner_config
        .get::<f64>(DefaultRetryConfig::KEY_BACKOFF_MULTIPLIER)
        .unwrap();
    assert_eq!(multiplier, 3.5);
}

// === Method chaining tests ===

#[test]
fn test_fluent_api() {
    let mut config = DefaultRetryConfig::new();

    config
        .set_max_attempts(8)
        .set_jitter_factor(0.15)
        .set_max_duration(Some(Duration::from_secs(120)))
        .set_fixed_delay_strategy(Duration::from_secs(5));

    assert_eq!(config.max_attempts(), 8);
    assert_eq!(config.jitter_factor(), 0.15);
    assert_eq!(config.max_duration(), Some(Duration::from_secs(120)));

    match config.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => {
            assert_eq!(delay, Duration::from_secs(5));
        }
        _ => panic!("Expected Fixed strategy"),
    }
}

// === Default trait test ===

#[test]
fn test_default_trait() {
    let config = DefaultRetryConfig::default();

    assert_eq!(config.max_attempts(), 5);
    assert_eq!(config.max_duration(), None);
    assert_eq!(config.jitter_factor(), 0.0);
}

// === Boundary condition tests ===

#[test]
fn test_max_attempts_boundary() {
    let mut config = DefaultRetryConfig::new();

    // Test minimum value
    config.set_max_attempts(1);
    assert_eq!(config.max_attempts(), 1);

    // Test large value
    config.set_max_attempts(1000);
    assert_eq!(config.max_attempts(), 1000);
}

#[test]
fn test_jitter_factor_boundary() {
    let mut config = DefaultRetryConfig::new();

    // Test zero value
    config.set_jitter_factor(0.0);
    assert_eq!(config.jitter_factor(), 0.0);

    // Test value 1
    config.set_jitter_factor(1.0);
    assert_eq!(config.jitter_factor(), 1.0);

    // Test value greater than 1
    config.set_jitter_factor(2.5);
    assert_eq!(config.jitter_factor(), 2.5);
}

#[test]
fn test_max_duration_zero() {
    let mut config = DefaultRetryConfig::new();

    // Setting to 0 should be equivalent to None (implementation converts 0 milliseconds to None)
    config.set_max_duration(Some(Duration::from_millis(0)));
    assert_eq!(config.max_duration(), None);
}

// === Clone test ===

#[test]
fn test_clone() {
    let mut config = DefaultRetryConfig::new();
    config.set_max_attempts(7);
    config.set_jitter_factor(0.25);
    config.set_fixed_delay_strategy(Duration::from_secs(4));

    let cloned_config = config.clone();

    assert_eq!(cloned_config.max_attempts(), 7);
    assert_eq!(cloned_config.jitter_factor(), 0.25);

    match cloned_config.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => {
            assert_eq!(delay, Duration::from_secs(4));
        }
        _ => panic!("Expected Fixed strategy"),
    }
}
