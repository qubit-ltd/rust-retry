/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Retry Reason Tests
//!
//! Tests for the RetryReason enum.

use prism3_retry::event::RetryReason;
use std::io;

/// Test RetryReason::Error variant
#[test]
fn test_retry_reason_error() {
    let error = Box::new(io::Error::new(io::ErrorKind::TimedOut, "Timeout"));
    let reason = RetryReason::<String>::Error(error);
    match reason {
        RetryReason::Error(err) => {
            assert!(err.to_string().contains("Timeout"));
        }
        _ => panic!("Expected Error variant"),
    }
}

/// Test RetryReason::Result variant
#[test]
fn test_retry_reason_result() {
    let reason = RetryReason::Result(42);
    match reason {
        RetryReason::Result(value) => {
            assert_eq!(value, 42);
        }
        _ => panic!("Expected Result variant"),
    }
}
