/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Single Operation Timeout Tests
//!
//! Tests various scenarios of operation_timeout functionality
//!
//! # Author
//!
//! Haixing Hu

use qubit_retry::{RetryBuilder, RetryError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn test_sync_operation_timeout_post_check_mechanism() {
    // Test sync version's post-check timeout mechanism
    let executor = RetryBuilder::<String>::new()
        .set_max_attempts(3)
        .set_operation_timeout(Some(Duration::from_millis(100)))
        .set_fixed_delay_strategy(Duration::from_millis(50))
        .build();

    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;

        // Simulate operation taking longer than timeout
        std::thread::sleep(Duration::from_millis(150));

        Ok::<String, Box<dyn std::error::Error + Send + Sync>>("SUCCESS".to_string())
    });

    // Should fail, as every operation times out
    assert!(result.is_err());
    match result {
        Err(RetryError::MaxAttemptsExceeded { attempts, .. }) => {
            assert_eq!(attempts, 3);
        }
        _ => panic!("Expected MaxAttemptsExceeded error"),
    }

    // Verify 3 attempts were made
    assert_eq!(*attempt_count.lock().unwrap(), 3);
}

#[test]
fn test_sync_operation_no_timeout() {
    // Test case without timeout limit
    let executor = RetryBuilder::<String>::new()
        .set_max_attempts(3)
        .set_unlimited_operation_timeout()
        .build();

    let result = executor.run(|| {
        // Simulate longer operation
        std::thread::sleep(Duration::from_millis(200));
        Ok::<String, Box<dyn std::error::Error + Send + Sync>>("SUCCESS".to_string())
    });

    // Should succeed, as there's no timeout limit
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "SUCCESS");
}

#[test]
fn test_sync_operation_within_timeout() {
    // Test operation completing within timeout
    let executor = RetryBuilder::<String>::new()
        .set_max_attempts(3)
        .set_operation_timeout(Some(Duration::from_millis(200)))
        .build();

    let result = executor.run(|| {
        // Operation completes quickly, within timeout
        std::thread::sleep(Duration::from_millis(50));
        Ok::<String, Box<dyn std::error::Error + Send + Sync>>("SUCCESS".to_string())
    });

    // Should succeed
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "SUCCESS");
}

#[test]
fn test_sync_operation_timeout_with_retry() {
    // Test retry after timeout, eventually succeeds
    let executor = RetryBuilder::<String>::new()
        .set_max_attempts(3)
        .set_operation_timeout(Some(Duration::from_millis(100)))
        .set_fixed_delay_strategy(Duration::from_millis(50))
        .build();

    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;

        if *count < 3 {
            // First two times timeout
            std::thread::sleep(Duration::from_millis(150));
        } else {
            // Third time completes quickly
            std::thread::sleep(Duration::from_millis(50));
        }

        Ok::<String, Box<dyn std::error::Error + Send + Sync>>("SUCCESS".to_string())
    });

    // Should succeed
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "SUCCESS");
    assert_eq!(*attempt_count.lock().unwrap(), 3);
}

#[tokio::test]
async fn test_async_operation_timeout_true_interruption() {
    // Test async version's true timeout interruption
    let executor = RetryBuilder::<String>::new()
        .set_max_attempts(3)
        .set_operation_timeout(Some(Duration::from_millis(100)))
        .set_fixed_delay_strategy(Duration::from_millis(50))
        .build();

    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let result = executor
        .run_async(|| {
            let attempt_count = attempt_count_clone.clone();
            async move {
                {
                    let mut count = attempt_count.lock().unwrap();
                    *count += 1;
                }

                // Simulate long-running operation (will be interrupted by timeout)
                tokio::time::sleep(Duration::from_millis(500)).await;

                Ok::<String, Box<dyn std::error::Error + Send + Sync>>("SUCCESS".to_string())
            }
        })
        .await;

    // Should fail
    assert!(result.is_err());
    match result {
        Err(RetryError::MaxAttemptsExceeded { attempts, .. }) => {
            assert_eq!(attempts, 3);
        }
        _ => panic!("Expected MaxAttemptsExceeded error"),
    }

    // Verify 3 attempts were made
    assert_eq!(*attempt_count.lock().unwrap(), 3);
}

#[tokio::test]
async fn test_async_operation_no_timeout() {
    // Test async version without timeout limit
    let executor = RetryBuilder::<String>::new()
        .set_max_attempts(3)
        .set_unlimited_operation_timeout()
        .build();

    let result = executor
        .run_async(|| async {
            // Simulate long-running operation
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok::<String, Box<dyn std::error::Error + Send + Sync>>("SUCCESS".to_string())
        })
        .await;

    // Should succeed
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "SUCCESS");
}

#[tokio::test]
async fn test_async_operation_within_timeout() {
    // Test async operation completing within timeout
    let executor = RetryBuilder::<String>::new()
        .set_max_attempts(3)
        .set_operation_timeout(Some(Duration::from_millis(200)))
        .build();

    let result = executor
        .run_async(|| async {
            // Completes quickly
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok::<String, Box<dyn std::error::Error + Send + Sync>>("SUCCESS".to_string())
        })
        .await;

    // Should succeed
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "SUCCESS");
}

#[tokio::test]
async fn test_async_operation_timeout_with_retry() {
    // Test async retry after timeout, eventually succeeds
    let executor = RetryBuilder::<String>::new()
        .set_max_attempts(3)
        .set_operation_timeout(Some(Duration::from_millis(100)))
        .set_fixed_delay_strategy(Duration::from_millis(50))
        .build();

    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let result = executor
        .run_async(|| {
            let attempt_count = attempt_count_clone.clone();
            async move {
                let current_attempt = {
                    let mut count = attempt_count.lock().unwrap();
                    *count += 1;
                    *count
                };

                if current_attempt < 3 {
                    // First two times timeout
                    tokio::time::sleep(Duration::from_millis(200)).await;
                } else {
                    // Third time completes quickly
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }

                Ok::<String, Box<dyn std::error::Error + Send + Sync>>("SUCCESS".to_string())
            }
        })
        .await;

    // Should succeed
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "SUCCESS");
    assert_eq!(*attempt_count.lock().unwrap(), 3);
}

#[tokio::test]
async fn test_async_max_duration_vs_operation_timeout() {
    // Test difference between max_duration and operation_timeout
    let executor = RetryBuilder::<String>::new()
        .set_max_attempts(10)
        .set_operation_timeout(Some(Duration::from_millis(100))) // Single operation max 100ms
        .set_max_duration(Some(Duration::from_millis(500))) // Total max 500ms
        .set_fixed_delay_strategy(Duration::from_millis(100))
        .build();

    let result = executor
        .run_async(|| async {
            // Every time times out
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok::<String, Box<dyn std::error::Error + Send + Sync>>("SUCCESS".to_string())
        })
        .await;

    // Should fail due to max_duration
    assert!(result.is_err());
    // Could be MaxDurationExceeded or MaxAttemptsExceeded, depending on which is reached first
}

#[test]
fn test_operation_timeout_config_from_file() {
    // Test loading operation_timeout from config file
    use qubit_config::Config;
    use qubit_retry::DefaultRetryConfig;

    let mut config = Config::new();
    config.set("retry.max_attempts", 3u32).unwrap();
    config
        .set("retry.operation_timeout_millis", 100u64)
        .unwrap();

    let retry_config = DefaultRetryConfig::with_config(config);
    let executor = RetryBuilder::<String>::with_config(retry_config).build();

    // Verify config loaded correctly
    assert_eq!(executor.config().max_attempts(), 3);
    assert_eq!(
        executor.config().operation_timeout(),
        Some(Duration::from_millis(100))
    );
}

#[test]
fn test_operation_timeout_event_listening() {
    // Test event triggered on timeout
    let retry_count = Arc::new(Mutex::new(0));
    let retry_count_clone = retry_count.clone();

    let executor = RetryBuilder::<String>::new()
        .set_max_attempts(3)
        .set_operation_timeout(Some(Duration::from_millis(100)))
        .set_fixed_delay_strategy(Duration::from_millis(50))
        .on_retry(move |_event: &qubit_retry::RetryEvent<String>| {
            let mut count = retry_count_clone.lock().unwrap();
            *count += 1;
        })
        .build();

    let _ = executor.run(|| {
        std::thread::sleep(Duration::from_millis(150));
        Ok::<String, Box<dyn std::error::Error + Send + Sync>>("SUCCESS".to_string())
    });

    // Should have triggered 2 retry events (retry after first failure, retry after second failure)
    assert_eq!(*retry_count.lock().unwrap(), 2);
}
