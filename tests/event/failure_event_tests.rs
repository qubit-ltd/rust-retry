/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Failure Event Tests
//!
//! Tests for the FailureEvent struct and builder.

use prism3_retry::event::FailureEvent;
use std::io;
use std::time::Duration;

/// Test FailureEvent creation and field access
#[test]
fn test_failure_event() {
    let error = Box::new(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    let event = FailureEvent::builder()
        .last_error(Some(error))
        .last_result(Some("FAILED".to_string()))
        .attempt_count(5)
        .total_duration(Duration::from_millis(5000))
        .build();

    assert!(event.last_error().is_some());
    assert_eq!(event.last_result(), Some(&"FAILED".to_string()));
    assert_eq!(event.attempt_count(), 5);
    assert_eq!(event.total_duration(), Duration::from_millis(5000));
}

/// Test FailureEvent with error only, no result
#[test]
fn test_failure_event_error_only() {
    let error = Box::new(io::Error::new(io::ErrorKind::TimedOut, "Timeout"));
    let event = FailureEvent::<String>::builder()
        .last_error(Some(error))
        .attempt_count(3)
        .total_duration(Duration::from_millis(3000))
        .build();

    assert!(event.last_error().is_some());
    assert!(event.last_result().is_none());
    assert!(event.last_error().unwrap().to_string().contains("Timeout"));
}

/// Test FailureEvent with result only, no error
#[test]
fn test_failure_event_result_only() {
    let event = FailureEvent::builder()
        .last_result(Some("INVALID_RESULT".to_string()))
        .attempt_count(4)
        .total_duration(Duration::from_millis(4000))
        .build();

    assert!(event.last_error().is_none());
    assert_eq!(event.last_result(), Some(&"INVALID_RESULT".to_string()));
}

/// Test FailureEvent with neither error nor result
#[test]
fn test_failure_event_empty() {
    let event = FailureEvent::<String>::builder()
        .attempt_count(5)
        .total_duration(Duration::from_millis(5000))
        .build();

    assert!(event.last_error().is_none());
    assert!(event.last_result().is_none());
    assert_eq!(event.attempt_count(), 5);
}

/// Test FailureEvent builder pattern
#[test]
fn test_failure_event_builder() {
    let event = FailureEvent::<String>::builder()
        .last_result(Some("FAILED".to_string()))
        .attempt_count(5)
        .total_duration(Duration::from_millis(5000))
        .build();

    assert_eq!(event.last_result(), Some(&"FAILED".to_string()));
    assert_eq!(event.attempt_count(), 5);
    assert_eq!(event.total_duration(), Duration::from_millis(5000));
}
