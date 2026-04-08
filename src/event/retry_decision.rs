/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Retry Decision
//!
//! Represents the decision on the current execution result.
//!
//! # Author
//!
//! Haixing Hu

use super::{abort_reason::AbortReason, retry_reason::RetryReason};

/// Retry decision enum
///
/// Represents the decision on the current execution result, used to
/// control the execution path of the retry flow.
///
/// # Characteristics
///
/// - `Success(T)`: Operation completed successfully, returns result
/// - `Retry(RetryReason<T>)`: Operation failed but can be retried
/// - `Abort(AbortReason<T>)`: Operation failed and should not
///   continue retrying
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Use Cases
///
/// Used in custom retry strategies to determine whether an operation
/// result needs to be retried or aborted.
///
/// # Example
///
/// ```rust
/// use qubit_retry::event::retry_decision::RetryDecision;
/// use qubit_retry::event::retry_reason::RetryReason;
/// use std::io::{Error, ErrorKind};
///
/// fn check_result(value: i32) -> RetryDecision<i32> {
///     if value > 0 {
///         RetryDecision::Success(value)
///     } else {
///         let error = Error::new(
///             ErrorKind::Other,
///             "Value must be greater than 0"
///         );
///         RetryDecision::Retry(RetryReason::Error(Box::new(error)))
///     }
/// }
/// ```
///
/// # Author
///
/// Haixing Hu
#[derive(Debug)]
pub enum RetryDecision<T> {
    /// Success, no need to retry
    Success(T),
    /// Need to retry
    Retry(RetryReason<T>),
    /// Need to abort
    Abort(AbortReason<T>),
}
