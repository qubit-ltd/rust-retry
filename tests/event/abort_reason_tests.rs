/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Abort Reason Tests
//!
//! Tests for the AbortReason enum.

use prism3_retry::event::AbortReason;
use std::io;

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
