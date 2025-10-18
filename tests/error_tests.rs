/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # RetryError Type Tests
//!
//! Tests various features and behaviors of RetryError.

use prism3_retry::{RetryError, RetryResult};
use std::error::Error;
use std::io;
use std::time::Duration;

/// Test RetryError's Display trait implementation
#[test]
fn test_retry_error_display() {
    let error = RetryError::max_attempts_exceeded(5, 3);
    assert!(error.to_string().contains("Maximum attempts exceeded"));
    assert!(error.to_string().contains("5"));
    assert!(error.to_string().contains("3"));
}

/// Test conversion from io::Error to RetryError
#[test]
fn test_retry_error_from_io_error() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let retry_error: RetryError = io_error.into();

    match retry_error {
        RetryError::ExecutionError { source } => {
            assert!(source.to_string().contains("File not found"));
        }
        _ => panic!("Expected ExecutionError"),
    }
}

/// Test creation and display of MaxAttemptsExceeded error
#[test]
fn test_max_attempts_exceeded() {
    let error = RetryError::max_attempts_exceeded(10, 5);
    let error_msg = error.to_string();
    assert!(error_msg.contains("Maximum attempts exceeded"));
    assert!(error_msg.contains("10"));
    assert!(error_msg.contains("5"));
}

/// Test creation and display of MaxDurationExceeded error
#[test]
fn test_max_duration_exceeded() {
    let duration = Duration::from_secs(10);
    let max_duration = Duration::from_secs(5);
    let error = RetryError::max_duration_exceeded(duration, max_duration);
    let error_msg = error.to_string();
    assert!(error_msg.contains("Maximum duration exceeded"));
    assert!(error_msg.contains("10s"));
    assert!(error_msg.contains("5s"));
}

/// Test creation and display of Aborted error
#[test]
fn test_aborted() {
    let error = RetryError::aborted("User cancelled operation");
    let error_msg = error.to_string();
    assert!(error_msg.contains("Operation aborted"));
    assert!(error_msg.contains("User cancelled operation"));
}

/// Test creation and display of ConfigError error
#[test]
fn test_config_error() {
    let error = RetryError::config_error("Maximum retry count cannot be negative");
    let error_msg = error.to_string();
    assert!(error_msg.contains("Configuration error"));
    assert!(error_msg.contains("Maximum retry count cannot be negative"));
}

/// Test creation and display of DelayStrategyError error
#[test]
fn test_delay_strategy_error() {
    let error = RetryError::delay_strategy_error("Delay time calculation overflow");
    let error_msg = error.to_string();
    assert!(error_msg.contains("Delay strategy error"));
    assert!(error_msg.contains("Delay time calculation overflow"));
}

/// Test creation and display of ExecutionError error
#[test]
fn test_execution_error() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let retry_error = RetryError::execution_error(io_error);
    let error_msg = retry_error.to_string();
    assert!(error_msg.contains("Execution error"));
    assert!(error_msg.contains("File not found"));
}

/// Test ExecutionError's source method
#[test]
fn test_execution_error_source() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let retry_error = RetryError::execution_error(io_error);

    let source = retry_error.source();
    assert!(source.is_some());
    assert!(source.unwrap().to_string().contains("File not found"));
}

/// Test non-ExecutionError source method returns None
#[test]
fn test_non_execution_error_source() {
    let error = RetryError::config_error("Configuration error");
    assert!(error.source().is_none());

    let error = RetryError::aborted("abort");
    assert!(error.source().is_none());

    let error = RetryError::other("Other error");
    assert!(error.source().is_none());
}

/// Test creating ExecutionError from Box<dyn Error>
#[test]
fn test_execution_error_box() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let boxed_error: Box<dyn Error + Send + Sync> = Box::new(io_error);
    let retry_error = RetryError::execution_error_box(boxed_error);

    let error_msg = retry_error.to_string();
    assert!(error_msg.contains("Execution error"));
    assert!(error_msg.contains("File not found"));
}

/// Test creation and display of Other error
#[test]
fn test_other_error() {
    let error = RetryError::other("Unknown error type");
    let error_msg = error.to_string();
    assert!(error_msg.contains("Other error"));
    assert!(error_msg.contains("Unknown error type"));
}

/// Test RetryResult type alias
#[test]
fn test_retry_result_success() {
    let result: RetryResult<String> = Ok("Operation successful".to_string());
    assert!(result.is_ok());
    if let Ok(value) = result {
        assert_eq!(value, "Operation successful");
    }
}

/// Test RetryResult failure case
#[test]
fn test_retry_result_failure() {
    let result: RetryResult<String> = Err(RetryError::other("Operation failed"));
    assert!(result.is_err());
    if let Err(error) = result {
        assert!(error.to_string().contains("Operation failed"));
    }
}

/// Test conversion from Box<dyn Error + Send + Sync> to RetryError
#[test]
fn test_from_boxed_error() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let boxed_error: Box<dyn Error + Send + Sync> = Box::new(io_error);
    let retry_error: RetryError = boxed_error.into();

    match retry_error {
        RetryError::ExecutionError { source } => {
            assert!(source.to_string().contains("File not found"));
        }
        _ => panic!("Expected ExecutionError"),
    }
}

/// Test using ? operator for io::Error conversion in functions
#[test]
fn test_io_error_conversion_with_question_mark() {
    fn io_operation() -> RetryResult<()> {
        let _file = std::fs::File::open("/non-existent-path/test.txt")?;
        Ok(())
    }

    let result = io_operation();
    assert!(result.is_err());
    let error = result.unwrap_err();
    match error {
        RetryError::ExecutionError { .. } => {
            // Verify error type is correct
        }
        _ => panic!("Expected ExecutionError"),
    }
}

/// Test Debug trait implementation
#[test]
fn test_debug_format() {
    let error = RetryError::max_attempts_exceeded(5, 3);
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("MaxAttemptsExceeded"));
    assert!(debug_str.contains("attempts"));
    assert!(debug_str.contains("max_attempts"));
}

/// Test Display format for all error types
#[test]
fn test_all_error_display_formats() {
    // MaxAttemptsExceeded
    let error = RetryError::MaxAttemptsExceeded {
        attempts: 5,
        max_attempts: 3,
    };
    assert!(error.to_string().contains("Maximum attempts exceeded"));

    // MaxDurationExceeded
    let error = RetryError::MaxDurationExceeded {
        duration: Duration::from_secs(10),
        max_duration: Duration::from_secs(5),
    };
    assert!(error.to_string().contains("Maximum duration exceeded"));

    // Aborted
    let error = RetryError::Aborted {
        reason: "Test abort".to_string(),
    };
    assert!(error.to_string().contains("Operation aborted"));

    // ConfigError
    let error = RetryError::ConfigError {
        message: "Test configuration error".to_string(),
    };
    assert!(error.to_string().contains("Configuration error"));

    // DelayStrategyError
    let error = RetryError::DelayStrategyError {
        message: "Test delay strategy error".to_string(),
    };
    assert!(error.to_string().contains("Delay strategy error"));

    // ExecutionError
    let io_error = io::Error::new(io::ErrorKind::NotFound, "Test file not found");
    let error = RetryError::ExecutionError {
        source: Box::new(io_error),
    };
    assert!(error.to_string().contains("Execution error"));

    // Other
    let error = RetryError::Other {
        message: "Test other error".to_string(),
    };
    assert!(error.to_string().contains("Other error"));
}

/// Test error chain tracking
#[test]
fn test_error_chain() {
    // Create a nested error chain
    let io_error = io::Error::new(io::ErrorKind::NotFound, "Original error");
    let retry_error = RetryError::execution_error(io_error);

    // Verify we can get the source error
    let source = retry_error.source();
    assert!(source.is_some());

    let source_error = source.unwrap();
    assert!(source_error.to_string().contains("Original error"));
}

/// Test using RetryResult in practical scenarios
#[test]
fn test_retry_result_in_practice() {
    fn simulate_retry_operation(should_fail: bool) -> RetryResult<i32> {
        if should_fail {
            Err(RetryError::max_attempts_exceeded(3, 3))
        } else {
            Ok(42)
        }
    }

    // Success case
    let result = simulate_retry_operation(false);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);

    // Failure case
    let result = simulate_retry_operation(true);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Maximum attempts exceeded"));
}

/// Test OperationTimeout error creation and display
#[test]
fn test_operation_timeout_error() {
    let duration = Duration::from_secs(10);
    let timeout = Duration::from_secs(5);
    let error = RetryError::operation_timeout(duration, timeout);
    let error_msg = error.to_string();
    assert!(error_msg.contains("Operation timeout"));
    assert!(error_msg.contains("10s"));
    assert!(error_msg.contains("5s"));
}

/// Test OperationTimeout error with enum constructor
#[test]
fn test_operation_timeout_enum_variant() {
    let error = RetryError::OperationTimeout {
        duration: Duration::from_secs(10),
        timeout: Duration::from_secs(5),
    };
    let error_msg = error.to_string();
    assert!(error_msg.contains("Operation timeout"));
    assert!(error_msg.contains("10s"));
    assert!(error_msg.contains("5s"));
}
