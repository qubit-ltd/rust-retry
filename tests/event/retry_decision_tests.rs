/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Retry Decision Tests
//!
//! Tests for the RetryDecision enum.

use prism3_retry::event::{AbortReason, RetryDecision, RetryReason};
use std::io;

/// Test RetryDecision::Success variant
#[test]
fn test_retry_decision_success() {
    let decision = RetryDecision::Success("success".to_string());
    match decision {
        RetryDecision::Success(result) => {
            assert_eq!(result, "success");
        }
        _ => panic!("Expected Success variant"),
    }
}

/// Test RetryDecision::Retry variant (error reason)
#[test]
fn test_retry_decision_retry_error() {
    let error = Box::new(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    let decision = RetryDecision::<String>::Retry(RetryReason::Error(error));
    match decision {
        RetryDecision::Retry(RetryReason::Error(err)) => {
            assert!(err.to_string().contains("File not found"));
        }
        _ => panic!("Expected Retry(Error) variant"),
    }
}

/// Test RetryDecision::Retry variant (result reason)
#[test]
fn test_retry_decision_retry_result() {
    let decision = RetryDecision::Retry(RetryReason::Result("retry".to_string()));
    match decision {
        RetryDecision::Retry(RetryReason::Result(result)) => {
            assert_eq!(result, "retry");
        }
        _ => panic!("Expected Retry(Result) variant"),
    }
}

/// Test RetryDecision::Abort variant (error reason)
#[test]
fn test_retry_decision_abort_error() {
    let error = Box::new(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "Access denied",
    ));
    let decision = RetryDecision::<String>::Abort(AbortReason::Error(error));
    match decision {
        RetryDecision::Abort(AbortReason::Error(err)) => {
            assert!(err.to_string().contains("Access denied"));
        }
        _ => panic!("Expected Abort(Error) variant"),
    }
}

/// Test RetryDecision::Abort variant (result reason)
#[test]
fn test_retry_decision_abort_result() {
    let decision = RetryDecision::Abort(AbortReason::Result("abort".to_string()));
    match decision {
        RetryDecision::Abort(AbortReason::Result(result)) => {
            assert_eq!(result, "abort");
        }
        _ => panic!("Expected Abort(Result) variant"),
    }
}
