/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Retry Event Tests
//!
//! Tests for the RetryEvent struct and builder.

use prism3_retry::event::RetryEvent;
use std::io;
use std::time::Duration;

/// Test RetryEvent creation and basic field access
#[test]
fn test_retry_event() {
    let error = Box::new(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    let event = RetryEvent::builder()
        .attempt_count(2)
        .max_attempts(5)
        .last_error(Some(error))
        .last_result(Some("FAILED".to_string()))
        .next_delay(Duration::from_secs(1))
        .total_duration(Duration::from_millis(1500))
        .build();

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
    let event = RetryEvent::<String>::builder()
        .attempt_count(5)
        .max_attempts(5)
        .next_delay(Duration::from_secs(1))
        .total_duration(Duration::from_millis(5000))
        .build();

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
    let event = RetryEvent::<String>::builder()
        .attempt_count(1)
        .max_attempts(3)
        .last_error(Some(error))
        .next_delay(Duration::from_millis(500))
        .total_duration(Duration::from_millis(1000))
        .build();

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
    let event = RetryEvent::builder()
        .attempt_count(1)
        .max_attempts(3)
        .last_result(Some(vec![1, 2, 3]))
        .next_delay(Duration::from_millis(500))
        .total_duration(Duration::from_millis(1000))
        .build();

    assert!(event.last_error().is_none());
    assert_eq!(event.last_result(), Some(&vec![1, 2, 3]));
}

/// Test RetryEvent first attempt
#[test]
fn test_retry_event_first_attempt() {
    let event = RetryEvent::<String>::builder()
        .attempt_count(1)
        .max_attempts(5)
        .next_delay(Duration::from_millis(100))
        .total_duration(Duration::from_millis(100))
        .build();

    assert_eq!(event.attempt_count(), 1);
    assert!(event.has_remaining_attempts());
}

/// Test RetryEvent builder pattern
#[test]
fn test_retry_event_builder() {
    let event = RetryEvent::<String>::builder()
        .attempt_count(2)
        .max_attempts(5)
        .next_delay(Duration::from_secs(1))
        .total_duration(Duration::from_millis(1500))
        .build();

    assert_eq!(event.attempt_count(), 2);
    assert_eq!(event.max_attempts(), 5);
    assert_eq!(event.next_delay(), Duration::from_secs(1));
    assert_eq!(event.total_duration(), Duration::from_millis(1500));
}
