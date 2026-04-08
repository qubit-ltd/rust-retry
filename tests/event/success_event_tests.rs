/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Success Event Tests
//!
//! Tests for the SuccessEvent struct and builder.

use qubit_retry::event::SuccessEvent;
use std::time::Duration;

/// Test SuccessEvent creation and field access
#[test]
fn test_success_event() {
    let event = SuccessEvent::builder()
        .result("SUCCESS".to_string())
        .attempt_count(3)
        .total_duration(Duration::from_millis(2000))
        .build();

    assert_eq!(event.result(), &"SUCCESS".to_string());
    assert_eq!(event.attempt_count(), 3);
    assert_eq!(event.total_duration(), Duration::from_millis(2000));
}

/// Test SuccessEvent success on first attempt
#[test]
fn test_success_event_first_attempt() {
    let event = SuccessEvent::builder()
        .result(42)
        .attempt_count(1)
        .total_duration(Duration::from_millis(100))
        .build();

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
    let event = SuccessEvent::builder()
        .result(result.clone())
        .attempt_count(2)
        .total_duration(Duration::from_millis(500))
        .build();

    assert_eq!(event.result(), &result);
    assert_eq!(event.attempt_count(), 2);
}

/// Test SuccessEvent builder pattern
#[test]
fn test_success_event_builder() {
    let event = SuccessEvent::builder()
        .result("SUCCESS".to_string())
        .attempt_count(3)
        .total_duration(Duration::from_millis(2000))
        .build();

    assert_eq!(event.result(), &"SUCCESS".to_string());
    assert_eq!(event.attempt_count(), 3);
    assert_eq!(event.total_duration(), Duration::from_millis(2000));
}
