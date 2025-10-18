/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Abort Reason
//!
//! Describes the specific reason why an operation should be aborted.
//!
//! # Author
//!
//! Haixing Hu

use std::error::Error as StdError;

/// Abort reason enum
///
/// Describes the specific reason why an operation should be aborted,
/// indicating situations where retrying should not continue.
///
/// # Characteristics
///
/// - `Error`: Needs abortion due to an unrecoverable error (e.g.,
///   permission error, resource does not exist)
/// - `Result`: The returned result indicates that retrying should
///   not continue (e.g., explicit rejection, invalid request)
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Use Cases
///
/// Identifies situations in retry strategies where retrying should
/// not occur, avoiding ineffective retry attempts.
///
/// # Example
///
/// ```rust
/// use prism3_retry::event::abort_reason::AbortReason;
/// use std::io::{Error, ErrorKind};
///
/// // Abort due to unrecoverable error
/// let error = Error::new(
///     ErrorKind::PermissionDenied,
///     "Permission denied"
/// );
/// let abort_by_error = AbortReason::<String>::Error(
///     Box::new(error)
/// );
///
/// // Abort due to explicit rejection result
/// let invalid_result = String::from("INVALID_REQUEST");
/// let abort_by_result = AbortReason::Result(invalid_result);
/// ```
///
/// # Author
///
/// Haixing Hu
#[derive(Debug)]
pub enum AbortReason<T> {
    /// Need to abort due to error
    Error(Box<dyn StdError + Send + Sync>),
    /// Need to abort due to result
    Result(T),
}
