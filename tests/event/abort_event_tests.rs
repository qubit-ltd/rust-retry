/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Abort Event Tests
//!
//! Tests for the AbortEvent struct and builder.

use qubit_retry::event::{AbortEvent, AbortReason};
use std::io;
use std::time::Duration;

/// Test AbortEvent creation and field access (error reason)
#[test]
fn test_abort_event() {
    let reason = AbortReason::<String>::Error(Box::new(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "Access denied",
    )));
    let event = AbortEvent::builder()
        .reason(reason)
        .attempt_count(2)
        .total_duration(Duration::from_millis(1000))
        .build();

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
    let event = AbortEvent::builder()
        .reason(reason)
        .attempt_count(1)
        .total_duration(Duration::from_millis(500))
        .build();

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
    let event = AbortEvent::builder()
        .reason(reason)
        .attempt_count(1)
        .total_duration(Duration::from_millis(100))
        .build();

    assert_eq!(event.attempt_count(), 1);
}

/// Test AbortEvent builder pattern
#[test]
fn test_abort_event_builder() {
    let reason = AbortReason::Result("INVALID".to_string());
    let event = AbortEvent::builder()
        .reason(reason)
        .attempt_count(2)
        .total_duration(Duration::from_millis(1000))
        .build();

    assert_eq!(event.attempt_count(), 2);
    assert_eq!(event.total_duration(), Duration::from_millis(1000));
    match event.reason() {
        AbortReason::Result(result) => {
            assert_eq!(result, "INVALID");
        }
        _ => panic!("Expected Result reason"),
    }
}
