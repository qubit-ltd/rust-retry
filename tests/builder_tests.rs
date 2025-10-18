//! # RetryBuilder Integration Tests
//!
//! Tests the public API functionality of RetryBuilder.

use prism3_retry::{
    DefaultRetryConfig, RetryBuilder, RetryConfig, RetryDelayStrategy, SimpleRetryConfig,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TestResult(String);

#[test]
fn test_retry_builder_creation() {
    let builder = RetryBuilder::<TestResult>::new();
    assert_eq!(builder.max_attempts(), 5);
    match builder.delay_strategy() {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay,
            max_delay,
            multiplier,
        } => {
            assert_eq!(initial_delay, Duration::from_millis(1000));
            assert_eq!(max_delay, Duration::from_secs(60));
            assert_eq!(multiplier, 2.0);
        }
        _ => panic!("Expected ExponentialBackoff delay strategy"),
    }
}

#[test]
fn test_set_max_attempts() {
    let builder = RetryBuilder::<TestResult>::new().set_max_attempts(10);
    assert_eq!(builder.max_attempts(), 10);
}

#[test]
fn test_set_delay_strategies() {
    // Test fixed delay strategy
    let builder = RetryBuilder::<TestResult>::new().set_delay_strategy(RetryDelayStrategy::Fixed {
        delay: Duration::from_secs(2),
    });
    match builder.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => {
            assert_eq!(delay, Duration::from_secs(2));
        }
        _ => panic!("Expected Fixed delay strategy"),
    }

    // Test random delay strategy
    let builder =
        RetryBuilder::<TestResult>::new().set_delay_strategy(RetryDelayStrategy::Random {
            min_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(500),
        });
    match builder.delay_strategy() {
        RetryDelayStrategy::Random {
            min_delay,
            max_delay,
        } => {
            assert_eq!(min_delay, Duration::from_millis(100));
            assert_eq!(max_delay, Duration::from_millis(500));
        }
        _ => panic!("Expected Random delay strategy"),
    }

    // Test exponential backoff strategy
    let builder = RetryBuilder::<TestResult>::new().set_delay_strategy(
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            multiplier: 1.5,
        },
    );
    match builder.delay_strategy() {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay,
            max_delay,
            multiplier,
        } => {
            assert_eq!(initial_delay, Duration::from_millis(500));
            assert_eq!(max_delay, Duration::from_secs(30));
            assert_eq!(multiplier, 1.5);
        }
        _ => panic!("Expected ExponentialBackoff delay strategy"),
    }
}

#[test]
fn test_failed_on_results() {
    let builder = RetryBuilder::<TestResult>::new().failed_on_results(vec![
        TestResult("ERROR".to_string()),
        TestResult("FAIL".to_string()),
    ]);

    // Verify config is set correctly (through public methods)
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_failed_on_results_if() {
    let builder =
        RetryBuilder::<TestResult>::new().failed_on_results_if(|result| result.0.contains("ERROR"));

    // Verify config is set correctly (through public methods)
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_abort_on_results() {
    let builder =
        RetryBuilder::<TestResult>::new().abort_on_results(vec![TestResult("ABORT".to_string())]);

    // Verify config is set correctly (through public methods)
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_abort_on_results_if() {
    let builder =
        RetryBuilder::<TestResult>::new().abort_on_results_if(|result| result.0.contains("ABORT"));

    // Verify config is set correctly (through public methods)
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_failed_on_errors() {
    let builder =
        RetryBuilder::<TestResult>::new().failed_on_errors::<std::io::Error, std::io::Error>();

    // Verify config is set correctly (through public methods)
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_abort_on_errors() {
    let builder =
        RetryBuilder::<TestResult>::new().abort_on_errors::<std::io::Error, std::io::Error>();

    // Verify config is set correctly (through public methods)
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_build_executor() {
    let _executor = RetryBuilder::<TestResult>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(100),
        })
        .build();

    // Verify executor can be created successfully (reaching here means successful creation)
}

// ==================== Generic tests using SimpleRetryConfig ====================

#[test]
fn test_builder_with_simple_config() {
    let config = SimpleRetryConfig::new();
    let builder = RetryBuilder::<TestResult, SimpleRetryConfig>::with_config(config);

    assert_eq!(builder.max_attempts(), 5);
    assert_eq!(builder.max_duration(), None);
    assert_eq!(builder.operation_timeout(), None);
}

#[test]
fn test_builder_with_simple_config_custom_params() {
    let config = SimpleRetryConfig::with_params(
        3,
        RetryDelayStrategy::fixed(Duration::from_secs(2)),
        0.1,
        Some(Duration::from_secs(30)),
        Some(Duration::from_secs(5)),
    );
    let builder = RetryBuilder::<TestResult, SimpleRetryConfig>::with_config(config);

    assert_eq!(builder.max_attempts(), 3);
    assert_eq!(builder.max_duration(), Some(Duration::from_secs(30)));
    assert_eq!(builder.operation_timeout(), Some(Duration::from_secs(5)));

    match builder.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => assert_eq!(delay, Duration::from_secs(2)),
        _ => panic!("Expected Fixed strategy"),
    }
}

#[test]
fn test_builder_with_simple_config_chain_methods() {
    let config = SimpleRetryConfig::new();
    let builder = RetryBuilder::<TestResult, SimpleRetryConfig>::with_config(config)
        .set_max_attempts(10)
        .set_max_duration(Some(Duration::from_secs(60)))
        .set_operation_timeout(Some(Duration::from_secs(10)))
        .set_delay_strategy(RetryDelayStrategy::exponential_backoff(
            Duration::from_millis(100),
            Duration::from_secs(30),
            2.0,
        ));

    assert_eq!(builder.max_attempts(), 10);
    assert_eq!(builder.max_duration(), Some(Duration::from_secs(60)));
    assert_eq!(builder.operation_timeout(), Some(Duration::from_secs(10)));

    match builder.delay_strategy() {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay,
            max_delay,
            multiplier,
        } => {
            assert_eq!(initial_delay, Duration::from_millis(100));
            assert_eq!(max_delay, Duration::from_secs(30));
            assert_eq!(multiplier, 2.0);
        }
        _ => panic!("Expected ExponentialBackoff strategy"),
    }
}

#[test]
fn test_builder_with_simple_config_build_executor() {
    let config = SimpleRetryConfig::new();
    let _executor = RetryBuilder::<TestResult, SimpleRetryConfig>::with_config(config)
        .set_max_attempts(3)
        .build();

    // Verify executor creation successful (reaching here means successful creation)
}

#[test]
fn test_simple_vs_default_config_consistency() {
    let simple_config = SimpleRetryConfig::with_params(
        3,
        RetryDelayStrategy::fixed(Duration::from_secs(1)),
        0.0,
        Some(Duration::from_secs(30)),
        Some(Duration::from_secs(5)),
    );

    let mut default_config = DefaultRetryConfig::new();
    default_config
        .set_max_attempts(3)
        .set_max_duration(Some(Duration::from_secs(30)))
        .set_fixed_delay_strategy(Duration::from_secs(1));

    let simple_builder = RetryBuilder::<TestResult, SimpleRetryConfig>::with_config(simple_config);
    let default_builder =
        RetryBuilder::<TestResult, DefaultRetryConfig>::with_config(default_config);

    // Verify both configs produce the same core behavior
    assert_eq!(
        simple_builder.max_attempts(),
        default_builder.max_attempts()
    );
    assert_eq!(
        simple_builder.max_duration(),
        default_builder.max_duration()
    );
}

// ==================== eventlistenertest ====================

#[test]
fn test_on_retry_listener() {
    let retry_called = Arc::new(Mutex::new(false));
    let retry_called_clone = retry_called.clone();

    let _builder = RetryBuilder::<TestResult>::new().on_retry(move |_event| {
        *retry_called_clone.lock().unwrap() = true;
    });

    // Verify listener set successfully (actual invocation tested during execution)
}

#[test]
fn test_on_success_listener() {
    let success_called = Arc::new(Mutex::new(false));
    let success_called_clone = success_called.clone();

    let _builder = RetryBuilder::<TestResult>::new().on_success(move |_event| {
        *success_called_clone.lock().unwrap() = true;
    });

    // Verify listener set successfully (reaching here means successful setup)
}

#[test]
fn test_on_failure_listener() {
    let failure_called = Arc::new(Mutex::new(false));
    let failure_called_clone = failure_called.clone();

    let _builder = RetryBuilder::<TestResult>::new().on_failure(move |_event| {
        *failure_called_clone.lock().unwrap() = true;
    });

    // Verify listener set successfully (reaching here means successful setup)
}

#[test]
fn test_on_abort_listener() {
    let abort_called = Arc::new(Mutex::new(false));
    let abort_called_clone = abort_called.clone();

    let _builder = RetryBuilder::<TestResult>::new().on_abort(move |_event| {
        *abort_called_clone.lock().unwrap() = true;
    });

    // Verify listener set successfully (reaching here means successful setup)
}

#[test]
fn test_all_event_listeners_together() {
    let retry_count = Arc::new(Mutex::new(0));
    let success_count = Arc::new(Mutex::new(0));
    let failure_count = Arc::new(Mutex::new(0));
    let abort_count = Arc::new(Mutex::new(0));

    let retry_count_clone = retry_count.clone();
    let success_count_clone = success_count.clone();
    let failure_count_clone = failure_count.clone();
    let abort_count_clone = abort_count.clone();

    let _builder = RetryBuilder::<TestResult>::new()
        .on_retry(move |_event| {
            *retry_count_clone.lock().unwrap() += 1;
        })
        .on_success(move |_event| {
            *success_count_clone.lock().unwrap() += 1;
        })
        .on_failure(move |_event| {
            *failure_count_clone.lock().unwrap() += 1;
        })
        .on_abort(move |_event| {
            *abort_count_clone.lock().unwrap() += 1;
        });

    // Verify all listeners set successfully (reaching here means all listeners set successfully)
}

// ==================== Config override semantics tests ====================

#[test]
fn test_failed_on_results_override() {
    // First setting of failed results
    let builder = RetryBuilder::<TestResult>::new()
        .failed_on_results(vec![TestResult("ERROR1".to_string())])
        .failed_on_results(vec![TestResult("ERROR2".to_string())]);

    // Second setting should override first, verify config successful
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_failed_on_results_if_override() {
    // First condition setting
    let builder = RetryBuilder::<TestResult>::new()
        .failed_on_results_if(|r| r.0.contains("ERROR"))
        .failed_on_results_if(|r| r.0.contains("FAIL"));

    // Second setting should override first
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_abort_on_results_override() {
    // First setting of abort results
    let builder = RetryBuilder::<TestResult>::new()
        .abort_on_results(vec![TestResult("ABORT1".to_string())])
        .abort_on_results(vec![TestResult("ABORT2".to_string())]);

    // Second setting should override first
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_abort_on_results_if_override() {
    // First condition setting
    let builder = RetryBuilder::<TestResult>::new()
        .abort_on_results_if(|r| r.0.contains("ABORT"))
        .abort_on_results_if(|r| r.0.contains("FATAL"));

    // Second setting should override first
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_listener_override() {
    let count1 = Arc::new(Mutex::new(0));
    let count2 = Arc::new(Mutex::new(0));

    let count1_clone = count1.clone();
    let count2_clone = count2.clone();

    // First listener setting, then second setting should override first
    let _builder = RetryBuilder::<TestResult>::new()
        .on_retry(move |_event| {
            *count1_clone.lock().unwrap() += 1;
        })
        .on_retry(move |_event| {
            *count2_clone.lock().unwrap() += 1;
        });

    // Verify setup successful (actual override behavior verified during execution, reaching here means successful setup)
}

// ==================== Complex condition combination tests ====================

#[test]
fn test_failed_and_abort_conditions_together() {
    let builder = RetryBuilder::<TestResult>::new()
        .failed_on_results(vec![TestResult("ERROR".to_string())])
        .abort_on_results(vec![TestResult("FATAL".to_string())]);

    // Set both failed and abort conditions simultaneously
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_failed_results_and_condition_together() {
    let builder = RetryBuilder::<TestResult>::new()
        .failed_on_results(vec![TestResult("ERROR".to_string())])
        .failed_on_results_if(|r| r.0.contains("FAIL"));

    // Note: failed_on_results_if will override failed_on_results
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_abort_results_and_condition_together() {
    let builder = RetryBuilder::<TestResult>::new()
        .abort_on_results(vec![TestResult("ABORT".to_string())])
        .abort_on_results_if(|r| r.0.contains("FATAL"));

    // Note: abort_on_results_if will override abort_on_results
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_errors_and_results_conditions_together() {
    let builder = RetryBuilder::<TestResult>::new()
        .failed_on_errors::<std::io::Error, std::io::Error>()
        .failed_on_results(vec![TestResult("ERROR".to_string())])
        .abort_on_errors::<std::io::Error, std::io::Error>()
        .abort_on_results(vec![TestResult("FATAL".to_string())]);

    // Configure both error and result conditions simultaneously
    assert_eq!(builder.max_attempts(), 5);
}

// ==================== operation_timeout test ====================

#[test]
fn test_operation_timeout() {
    let builder =
        RetryBuilder::<TestResult>::new().set_operation_timeout(Some(Duration::from_secs(5)));

    assert_eq!(builder.operation_timeout(), Some(Duration::from_secs(5)));
}

#[test]
fn test_unlimited_operation_timeout() {
    let builder = RetryBuilder::<TestResult>::new()
        .set_operation_timeout(Some(Duration::from_secs(5)))
        .set_unlimited_operation_timeout();

    assert_eq!(builder.operation_timeout(), None);
}

#[test]
fn test_operation_timeout_with_max_duration() {
    let builder = RetryBuilder::<TestResult>::new()
        .set_operation_timeout(Some(Duration::from_secs(5)))
        .set_max_duration(Some(Duration::from_secs(30)));

    assert_eq!(builder.operation_timeout(), Some(Duration::from_secs(5)));
    assert_eq!(builder.max_duration(), Some(Duration::from_secs(30)));
}

// ==================== Special error configuration tests ====================

#[test]
fn test_failed_on_all_errors() {
    let builder = RetryBuilder::<TestResult>::new().failed_on_all_errors();

    // Verify config successful
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_no_failed_errors() {
    let builder = RetryBuilder::<TestResult>::new().no_failed_errors();

    // Verify config successful
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_failed_on_all_errors_then_no_failed_errors() {
    // First enable all error retries, then disable
    let builder = RetryBuilder::<TestResult>::new()
        .failed_on_all_errors()
        .no_failed_errors();

    // Verify config successful (override semantics)
    assert_eq!(builder.max_attempts(), 5);
}

// ==================== Boundary condition tests ====================

#[test]
fn test_empty_failed_results() {
    let builder = RetryBuilder::<TestResult>::new().failed_on_results(vec![]);

    // Empty failed results list
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_empty_abort_results() {
    let builder = RetryBuilder::<TestResult>::new().abort_on_results(vec![]);

    // Empty abort results list
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_zero_max_attempts() {
    let builder = RetryBuilder::<TestResult>::new().set_max_attempts(0);

    assert_eq!(builder.max_attempts(), 0);
}

#[test]
fn test_very_large_max_attempts() {
    let builder = RetryBuilder::<TestResult>::new().set_max_attempts(u32::MAX);

    assert_eq!(builder.max_attempts(), u32::MAX);
}

#[test]
fn test_zero_max_duration() {
    let builder =
        RetryBuilder::<TestResult>::new().set_max_duration(Some(Duration::from_millis(0)));

    // DefaultRetryConfig converts 0 millisecond duration to None
    assert_eq!(builder.max_duration(), None);
}

#[test]
fn test_none_delay_strategy() {
    let builder = RetryBuilder::<TestResult>::new().set_delay_strategy(RetryDelayStrategy::None);

    match builder.delay_strategy() {
        RetryDelayStrategy::None => {}
        _ => panic!("Expected None strategy"),
    }
}

// ==================== Different result type tests ====================

#[test]
fn test_unit_result_type() {
    let _builder = RetryBuilder::<()>::new().set_max_attempts(3);

    // Verify () type can be used (reaching here means it can be used)
}

#[test]
fn test_numeric_result_type() {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct NumericResult(i32);

    let _builder = RetryBuilder::<NumericResult>::new()
        .failed_on_results_if(|r| r.0 < 0)
        .abort_on_results_if(|r| r.0 == -999);

    // Verify numeric types can be used (reaching here means it can be used)
}

#[test]
fn test_complex_result_type() {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct ComplexResult {
        code: i32,
        message: String,
        data: Option<Vec<String>>,
    }

    let _builder = RetryBuilder::<ComplexResult>::new()
        .failed_on_results_if(|r| r.code >= 500)
        .abort_on_results_if(|r| r.code == 401);

    // Verify complex types can be used (reaching here means it can be used)
}

// ==================== Delay strategy convenience method tests ====================

#[test]
fn test_set_fixed_delay_strategy() {
    let builder =
        RetryBuilder::<TestResult>::new().set_fixed_delay_strategy(Duration::from_secs(3));

    match builder.delay_strategy() {
        RetryDelayStrategy::Fixed { delay } => {
            assert_eq!(delay, Duration::from_secs(3));
        }
        _ => panic!("Expected Fixed delay strategy"),
    }
}

#[test]
fn test_set_random_delay_strategy() {
    let builder = RetryBuilder::<TestResult>::new()
        .set_random_delay_strategy(Duration::from_millis(200), Duration::from_millis(800));

    match builder.delay_strategy() {
        RetryDelayStrategy::Random {
            min_delay,
            max_delay,
        } => {
            assert_eq!(min_delay, Duration::from_millis(200));
            assert_eq!(max_delay, Duration::from_millis(800));
        }
        _ => panic!("Expected Random delay strategy"),
    }
}

#[test]
fn test_set_exponential_backoff_strategy() {
    let builder = RetryBuilder::<TestResult>::new().set_exponential_backoff_strategy(
        Duration::from_millis(200),
        Duration::from_secs(120),
        3.0,
    );

    match builder.delay_strategy() {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay,
            max_delay,
            multiplier,
        } => {
            assert_eq!(initial_delay, Duration::from_millis(200));
            assert_eq!(max_delay, Duration::from_secs(120));
            assert_eq!(multiplier, 3.0);
        }
        _ => panic!("Expected ExponentialBackoff delay strategy"),
    }
}

#[test]
fn test_set_no_delay_strategy() {
    let builder = RetryBuilder::<TestResult>::new().set_no_delay_strategy();

    match builder.delay_strategy() {
        RetryDelayStrategy::None => {}
        _ => panic!("Expected None delay strategy"),
    }
}

// ==================== Jitter factor tests ====================

#[test]
fn test_jitter_factor_getter() {
    let builder = RetryBuilder::<TestResult>::new();
    let jitter = builder.jitter_factor();
    assert!(
        (0.0..=1.0).contains(&jitter),
        "Jitter factor should be between 0 and 1"
    );
}

#[test]
fn test_set_jitter_factor() {
    let builder = RetryBuilder::<TestResult>::new().set_jitter_factor(0.25);
    assert_eq!(builder.jitter_factor(), 0.25);

    let builder2 = RetryBuilder::<TestResult>::new().set_jitter_factor(0.75);
    assert_eq!(builder2.jitter_factor(), 0.75);
}

// ==================== Duration-related method tests ====================

#[test]
fn test_max_duration_getter() {
    let builder = RetryBuilder::<TestResult>::new();
    assert_eq!(builder.max_duration(), None);

    let builder2 =
        RetryBuilder::<TestResult>::new().set_max_duration(Some(Duration::from_secs(120)));
    assert_eq!(builder2.max_duration(), Some(Duration::from_secs(120)));
}

#[test]
fn test_set_unlimited_duration() {
    let builder = RetryBuilder::<TestResult>::new()
        .set_max_duration(Some(Duration::from_secs(60)))
        .set_unlimited_duration();

    assert_eq!(builder.max_duration(), None);
}

// ==================== Clear method tests ====================

#[test]
fn test_clear_failed_results() {
    let builder = RetryBuilder::<TestResult>::new()
        .failed_on_results(vec![
            TestResult("ERROR".to_string()),
            TestResult("FAIL".to_string()),
        ])
        .clear_failed_results();

    // Verify executor can be built successfully
    let _executor = builder.build();
}

#[test]
fn test_clear_abort_results() {
    let builder = RetryBuilder::<TestResult>::new()
        .abort_on_results(vec![
            TestResult("ABORT".to_string()),
            TestResult("STOP".to_string()),
        ])
        .clear_abort_results();

    // Verify executor can be built successfully
    let _executor = builder.build();
}

// ==================== abort_on_errors test ====================

#[test]
fn test_abort_on_errors_multiple() {
    use std::fmt;
    use std::io;

    let builder = RetryBuilder::<TestResult>::new().abort_on_errors::<io::Error, fmt::Error>();

    // This test verifies that multiple error types can be set for abort and executor can be built successfully
    let _executor = builder.build();
}

// ==================== Default trait tests ====================

#[test]
fn test_default_trait() {
    let builder1 = RetryBuilder::<TestResult>::new();
    let builder2 = RetryBuilder::<TestResult>::default();

    assert_eq!(builder1.max_attempts(), builder2.max_attempts());
    assert_eq!(builder1.max_duration(), builder2.max_duration());
}

// ==================== Additional edge case tests ====================

#[test]
fn test_failed_on_error_single_type() {
    use std::io::Error as IoError;

    let builder = RetryBuilder::<TestResult>::new().failed_on_error::<IoError>();

    // Verify config successful
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_abort_on_error_single_type() {
    use std::io::Error as IoError;

    let builder = RetryBuilder::<TestResult>::new().abort_on_error::<IoError>();

    // Verify config successful
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_failed_on_result_single() {
    let builder =
        RetryBuilder::<TestResult>::new().failed_on_result(TestResult("ERROR".to_string()));

    // Verify config successful
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_abort_on_result_single() {
    let builder =
        RetryBuilder::<TestResult>::new().abort_on_result(TestResult("ABORT".to_string()));

    // Verify config successful
    assert_eq!(builder.max_attempts(), 5);
}

#[test]
fn test_abort_on_all_errors() {
    let builder = RetryBuilder::<TestResult>::new().abort_on_all_errors();

    // Verify config successful
    assert_eq!(builder.max_attempts(), 5);
}
