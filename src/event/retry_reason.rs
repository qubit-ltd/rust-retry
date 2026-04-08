/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Retry Reason
//!
//! Describes the specific reason why an operation needs to be retried.
//!
//! # Author
//!
//! Haixing Hu

use std::error::Error as StdError;

/// Retry reason enum
///
/// Describes the specific reason why an operation needs to be
/// retried, either due to an error or because the returned result
/// does not meet expectations.
///
/// # Characteristics
///
/// - `Error`: Needs retry due to an error (e.g., network exception,
///   timeout)
/// - `Result`: Needs retry because the returned result does not meet
///   expectations (e.g., empty return value, incomplete data)
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Use Cases
///
/// Distinguishes different failure reasons in retry strategies to
/// adopt different retry approaches.
///
/// # Example
///
/// ```rust
/// use qubit_retry::event::retry_reason::RetryReason;
/// use std::io::{Error, ErrorKind};
///
/// // Retry due to error
/// let error = Error::new(
///     ErrorKind::ConnectionRefused,
///     "Connection refused"
/// );
/// let retry_by_error = RetryReason::<String>::Error(
///     Box::new(error)
/// );
///
/// // Retry due to result
/// let empty_result = String::new();
/// let retry_by_result = RetryReason::Result(empty_result);
/// ```
///
/// # Author
///
/// Haixing Hu
#[derive(Debug)]
pub enum RetryReason<T> {
    /// Need to retry due to error
    Error(Box<dyn StdError + Send + Sync>),
    /// Need to retry due to result
    Result(T),
}
