/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # retryeventtest
//!
//! Tests various features and behaviors of retry event types.

use prism3_retry::events::{
    AbortEvent, AbortReason, FailureEvent, RetryDecision, RetryEvent, RetryReason, SuccessEvent,
};
use std::io;
use std::time::Duration;

// ============================================================================
// RetryDecision test
// ============================================================================

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

// ============================================================================
// RetryReason test
// ============================================================================

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

// ============================================================================
// AbortReason test
// ============================================================================

/// Test AbortReason::Error variant
#[test]
fn test_abort_reason_error() {
    let error = Box::new(io::Error::new(io::ErrorKind::InvalidInput, "Invalid input"));
    let reason = AbortReason::<String>::Error(error);
    match reason {
        AbortReason::Error(err) => {
            assert!(err.to_string().contains("Invalid input"));
        }
        _ => panic!("Expected Error variant"),
    }
}

/// Test AbortReason::Result variant
#[test]
fn test_abort_reason_result() {
    let reason = AbortReason::Result(vec![1, 2, 3]);
    match reason {
        AbortReason::Result(value) => {
            assert_eq!(value, vec![1, 2, 3]);
        }
        _ => panic!("Expected Result variant"),
    }
}

// ============================================================================
// RetryEvent test
// ============================================================================

/// Test RetryEvent creation and basic field access
#[test]
fn test_retry_event() {
    let error = Box::new(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    let event = RetryEvent::new(
        2,
        5,
        Some(error),
        Some("FAILED".to_string()),
        Duration::from_secs(1),
        Duration::from_millis(1500),
    );

    assert_eq!(event.attempt_count(), 2);
    assert_eq!(event.max_attempts(), 5);
    assert!(event.last_error().is_some());
    assert_eq!(event.last_result(), Some(&"FAILED".to_string()));
    assert_eq!(event.next_delay(), Duration::from_secs(1));
    assert_eq!(event.total_duration(), Duration::from_millis(1500));
    assert!(event.has_remaining_attempts());
}

/// Test RetryEvent with no remaining attempts
#[test]
fn test_retry_event_no_remaining_attempts() {
    let event = RetryEvent::<String>::new(
        5,
        5,
        None,
        None,
        Duration::from_secs(1),
        Duration::from_millis(5000),
    );

    assert_eq!(event.attempt_count(), 5);
    assert_eq!(event.max_attempts(), 5);
    assert!(!event.has_remaining_attempts());
}

/// Test RetryEvent with error only, no result
#[test]
fn test_retry_event_error_only() {
    let error = Box::new(io::Error::new(
        io::ErrorKind::ConnectionRefused,
        "Connection refused",
    ));
    let event = RetryEvent::<String>::new(
        1,
        3,
        Some(error),
        None,
        Duration::from_millis(500),
        Duration::from_millis(1000),
    );

    assert!(event.last_error().is_some());
    assert!(event.last_result().is_none());
    assert!(event
        .last_error()
        .unwrap()
        .to_string()
        .contains("Connection refused"));
}

/// Test RetryEvent with result only, no error
#[test]
fn test_retry_event_result_only() {
    let event = RetryEvent::new(
        1,
        3,
        None,
        Some(vec![1, 2, 3]),
        Duration::from_millis(500),
        Duration::from_millis(1000),
    );

    assert!(event.last_error().is_none());
    assert_eq!(event.last_result(), Some(&vec![1, 2, 3]));
}

/// Test RetryEvent first attempt
#[test]
fn test_retry_event_first_attempt() {
    let event = RetryEvent::<String>::new(
        1,
        5,
        None,
        None,
        Duration::from_millis(100),
        Duration::from_millis(100),
    );

    assert_eq!(event.attempt_count(), 1);
    assert!(event.has_remaining_attempts());
}

// ============================================================================
// SuccessEvent test
// ============================================================================

/// Test SuccessEvent creation and field access
#[test]
fn test_success_event() {
    let event = SuccessEvent::new("SUCCESS".to_string(), 3, Duration::from_millis(2000));

    assert_eq!(event.result(), &"SUCCESS".to_string());
    assert_eq!(event.attempt_count(), 3);
    assert_eq!(event.total_duration(), Duration::from_millis(2000));
}

/// Test SuccessEvent success on first attempt
#[test]
fn test_success_event_first_attempt() {
    let event = SuccessEvent::new(42, 1, Duration::from_millis(100));

    assert_eq!(event.result(), &42);
    assert_eq!(event.attempt_count(), 1);
}

/// Test SuccessEvent with complex result type
#[test]
fn test_success_event_complex_result() {
    let result = vec![
        ("key1".to_string(), 1),
        ("key2".to_string(), 2),
        ("key3".to_string(), 3),
    ];
    let event = SuccessEvent::new(result.clone(), 2, Duration::from_millis(500));

    assert_eq!(event.result(), &result);
    assert_eq!(event.attempt_count(), 2);
}

// ============================================================================
// FailureEvent test
// ============================================================================

/// Test FailureEvent creation and field access
#[test]
fn test_failure_event() {
    let error = Box::new(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    let event = FailureEvent::new(
        Some(error),
        Some("FAILED".to_string()),
        5,
        Duration::from_millis(5000),
    );

    assert!(event.last_error().is_some());
    assert_eq!(event.last_result(), Some(&"FAILED".to_string()));
    assert_eq!(event.attempt_count(), 5);
    assert_eq!(event.total_duration(), Duration::from_millis(5000));
}

/// Test FailureEvent with error only, no result
#[test]
fn test_failure_event_error_only() {
    let error = Box::new(io::Error::new(io::ErrorKind::TimedOut, "Timeout"));
    let event = FailureEvent::<String>::new(Some(error), None, 3, Duration::from_millis(3000));

    assert!(event.last_error().is_some());
    assert!(event.last_result().is_none());
    assert!(event.last_error().unwrap().to_string().contains("Timeout"));
}

/// Test FailureEvent with result only, no error
#[test]
fn test_failure_event_result_only() {
    let event = FailureEvent::new(
        None,
        Some("INVALID_RESULT".to_string()),
        4,
        Duration::from_millis(4000),
    );

    assert!(event.last_error().is_none());
    assert_eq!(event.last_result(), Some(&"INVALID_RESULT".to_string()));
}

/// Test FailureEvent with neither error nor result
#[test]
fn test_failure_event_empty() {
    let event = FailureEvent::<String>::new(None, None, 5, Duration::from_millis(5000));

    assert!(event.last_error().is_none());
    assert!(event.last_result().is_none());
    assert_eq!(event.attempt_count(), 5);
}

// ============================================================================
// AbortEvent test
// ============================================================================

/// Test AbortEvent creation and field access (error reason)
#[test]
fn test_abort_event() {
    let reason = AbortReason::<String>::Error(Box::new(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "Access denied",
    )));
    let event = AbortEvent::new(reason, 2, Duration::from_millis(1000));

    assert_eq!(event.attempt_count(), 2);
    assert_eq!(event.total_duration(), Duration::from_millis(1000));
    match event.reason() {
        AbortReason::Error(err) => {
            assert!(err.to_string().contains("Access denied"));
        }
        _ => panic!("Expected Error reason"),
    }
}

/// Test AbortEvent with result reason
#[test]
fn test_abort_event_result_reason() {
    let reason = AbortReason::Result("INVALID".to_string());
    let event = AbortEvent::new(reason, 1, Duration::from_millis(500));

    assert_eq!(event.attempt_count(), 1);
    match event.reason() {
        AbortReason::Result(result) => {
            assert_eq!(result, "INVALID");
        }
        _ => panic!("Expected Result reason"),
    }
}

/// Test AbortEvent abort on first attempt
#[test]
fn test_abort_event_first_attempt() {
    let reason = AbortReason::<String>::Error(Box::new(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Invalid input",
    )));
    let event = AbortEvent::new(reason, 1, Duration::from_millis(100));

    assert_eq!(event.attempt_count(), 1);
}

// ============================================================================
// Integration tests
// ============================================================================

/// Test complete retry event flow simulation
#[test]
fn test_event_flow_simulation() {
    // First attempt - failed
    let retry_event1: RetryEvent<String> = RetryEvent::new(
        1,
        3,
        Some(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "Not found",
        ))),
        None,
        Duration::from_millis(100),
        Duration::from_millis(100),
    );
    assert!(retry_event1.has_remaining_attempts());

    // Second attempt - failed
    let retry_event2: RetryEvent<String> = RetryEvent::new(
        2,
        3,
        Some(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "Not found",
        ))),
        None,
        Duration::from_millis(200),
        Duration::from_millis(300),
    );
    assert!(retry_event2.has_remaining_attempts());

    // Third attempt - success
    let success_event = SuccessEvent::new("OK".to_string(), 3, Duration::from_millis(500));
    assert_eq!(success_event.attempt_count(), 3);
    assert_eq!(success_event.result(), &"OK".to_string());
}

/// Test scenario where all attempts fail
#[test]
fn test_all_attempts_failed() {
    let max_attempts = 3;

    for attempt in 1..=max_attempts {
        let retry_event: RetryEvent<String> = RetryEvent::new(
            attempt,
            max_attempts,
            Some(Box::new(io::Error::new(
                io::ErrorKind::ConnectionRefused,
                "Connection refused",
            ))),
            None,
            Duration::from_millis(100 * attempt as u64),
            Duration::from_millis(100 * attempt as u64),
        );

        if attempt < max_attempts {
            assert!(retry_event.has_remaining_attempts());
        } else {
            assert!(!retry_event.has_remaining_attempts());
        }
    }

    // Final failure
    let failure_event = FailureEvent::<String>::new(
        Some(Box::new(io::Error::new(
            io::ErrorKind::ConnectionRefused,
            "Connection refused",
        ))),
        None,
        max_attempts,
        Duration::from_millis(300),
    );
    assert_eq!(failure_event.attempt_count(), max_attempts);
}

/// Test scenario where abort occurs on first attempt
#[test]
fn test_immediate_abort() {
    let abort_reason = AbortReason::<String>::Error(Box::new(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "Permission denied",
    )));
    let abort_event = AbortEvent::new(abort_reason, 1, Duration::from_millis(50));

    assert_eq!(abort_event.attempt_count(), 1);
    match abort_event.reason() {
        AbortReason::Error(err) => {
            assert!(err.to_string().contains("Permission denied"));
        }
        _ => panic!("Expected Error reason"),
    }
}
