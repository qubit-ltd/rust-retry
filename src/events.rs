/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Retry Event Types
//!
//! Defines various event types that can occur during the retry process.
//!
//! # Author
//!
//! Haixing Hu

use std::error::Error as StdError;
use std::time::Duration;

/// Retry decision enum
///
/// Represents the decision on the current execution result, used to control the execution path of the retry flow.
///
/// # Characteristics
///
/// - `Success(T)`: Operation completed successfully, returns result
/// - `Retry(RetryReason<T>)`: Operation failed but can be retried
/// - `Abort(AbortReason<T>)`: Operation failed and should not continue retrying
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Use Cases
///
/// Used in custom retry strategies to determine whether an operation result needs to be retried or aborted.
///
/// # Example
///
/// ```rust
/// use prism3_retry::events::{RetryDecision, RetryReason};
/// use std::io::{Error, ErrorKind};
///
/// fn check_result(value: i32) -> RetryDecision<i32> {
///     if value > 0 {
///         RetryDecision::Success(value)
///     } else {
///         let error = Error::new(ErrorKind::Other, "Value must be greater than 0");
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

/// Retry reason enum
///
/// Describes the specific reason why an operation needs to be retried, either due to an error or because the returned result does not meet expectations.
///
/// # Characteristics
///
/// - `Error`: Needs retry due to an error (e.g., network exception, timeout)
/// - `Result`: Needs retry because the returned result does not meet expectations (e.g., empty return value, incomplete data)
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Use Cases
///
/// Distinguishes different failure reasons in retry strategies to adopt different retry approaches.
///
/// # Example
///
/// ```rust
/// use prism3_retry::events::RetryReason;
/// use std::io::{Error, ErrorKind};
///
/// // Retry due to error
/// let error = Error::new(ErrorKind::ConnectionRefused, "Connection refused");
/// let retry_by_error = RetryReason::<String>::Error(Box::new(error));
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

/// Abort reason enum
///
/// Describes the specific reason why an operation should be aborted, indicating situations where retrying should not continue.
///
/// # Characteristics
///
/// - `Error`: Needs abortion due to an unrecoverable error (e.g., permission error, resource does not exist)
/// - `Result`: The returned result indicates that retrying should not continue (e.g., explicit rejection, invalid request)
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Use Cases
///
/// Identifies situations in retry strategies where retrying should not occur, avoiding ineffective retry attempts.
///
/// # Example
///
/// ```rust
/// use prism3_retry::events::AbortReason;
/// use std::io::{Error, ErrorKind};
///
/// // Abort due to unrecoverable error
/// let error = Error::new(ErrorKind::PermissionDenied, "Permission denied");
/// let abort_by_error = AbortReason::<String>::Error(Box::new(error));
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

/// Retry event
///
/// Event triggered when an operation fails and is preparing to retry, containing detailed information about the retry.
///
/// # Characteristics
///
/// - Records current attempt count and maximum attempts
/// - Saves the error or result from the last failure
/// - Contains the delay time for the next retry
/// - Tracks total execution time
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Use Cases
///
/// Used in retry listeners for logging, sending monitoring data, or implementing custom retry logic.
///
/// # Example
///
/// ```rust
/// use prism3_retry::events::RetryEvent;
/// use std::time::Duration;
///
/// let event = RetryEvent::<String>::new(
///     1,
///     3,
///     None,
///     Some(String::from("empty")),
///     Duration::from_secs(1),
///     Duration::from_millis(100),
/// );
///
/// println!("Retry attempt {} of {}, {} attempts remaining",
///     event.attempt_count(),
///     event.max_attempts(),
///     event.max_attempts() - event.attempt_count()
/// );
/// ```
///
/// # Author
///
/// Haixing Hu
#[derive(Debug)]
pub struct RetryEvent<T> {
    /// Current attempt count
    attempt_count: u32,
    /// Maximum attempt count
    max_attempts: u32,
    /// Reason for last failure
    last_error: Option<Box<dyn StdError + Send + Sync>>,
    /// Result from last failure
    last_result: Option<T>,
    /// Delay time for next retry
    next_delay: Duration,
    /// Total execution time
    total_duration: Duration,
}

impl<T> RetryEvent<T> {
    /// Create retry event
    ///
    /// # Parameters
    ///
    /// * `attempt_count` - Current attempt count
    /// * `max_attempts` - Maximum attempt count
    /// * `last_error` - Error from last failure (if any)
    /// * `last_result` - Result from last failure (if any)
    /// * `next_delay` - Delay time for next retry
    /// * `total_duration` - Total execution time
    ///
    /// # Returns
    ///
    /// Returns a newly created `RetryEvent` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let event = RetryEvent::<i32>::new(
    ///     1,
    ///     3,
    ///     None,
    ///     Some(0),
    ///     Duration::from_secs(1),
    ///     Duration::from_millis(100),
    /// );
    /// ```
    pub fn new(
        attempt_count: u32,
        max_attempts: u32,
        last_error: Option<Box<dyn StdError + Send + Sync>>,
        last_result: Option<T>,
        next_delay: Duration,
        total_duration: Duration,
    ) -> Self {
        Self {
            attempt_count,
            max_attempts,
            last_error,
            last_result,
            next_delay,
            total_duration,
        }
    }

    /// Get current attempt count
    ///
    /// # Returns
    ///
    /// Returns the current number of attempts already made（counting from 1）
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let event = RetryEvent::<i32>::new(2, 3, None, None, Duration::from_secs(1), Duration::from_millis(200));
    /// assert_eq!(event.attempt_count(), 2);
    /// ```
    pub fn attempt_count(&self) -> u32 {
        self.attempt_count
    }

    /// Get maximum attempt count
    ///
    /// # Returns
    ///
    /// Returns the maximum number of attempts allowed
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let event = RetryEvent::<i32>::new(1, 5, None, None, Duration::from_secs(1), Duration::from_millis(100));
    /// assert_eq!(event.max_attempts(), 5);
    /// ```
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    /// Get error from last failure
    ///
    /// # Returns
    ///
    /// Returns reference to the error if last failure was due to error, None otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::RetryEvent;
    /// use std::time::Duration;
    /// use std::io::{Error, ErrorKind};
    ///
    /// let error = Error::new(ErrorKind::TimedOut, "Timeout");
    /// let event = RetryEvent::<i32>::new(1, 3, Some(Box::new(error)), None, Duration::from_secs(1), Duration::from_millis(100));
    /// assert!(event.last_error().is_some());
    /// ```
    pub fn last_error(&self) -> Option<&(dyn StdError + Send + Sync)> {
        self.last_error.as_ref().map(|e| e.as_ref())
    }

    /// Get result from last failure
    ///
    /// # Returns
    ///
    /// Returns reference to the result if last failure had a return value, None otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let event = RetryEvent::new(1, 3, None, Some(String::from("empty")), Duration::from_secs(1), Duration::from_millis(100));
    /// assert_eq!(event.last_result(), Some(&String::from("empty")));
    /// ```
    pub fn last_result(&self) -> Option<&T> {
        self.last_result.as_ref()
    }

    /// Get delay time for next retry
    ///
    /// # Returns
    ///
    /// Returns the duration to wait before the next retry
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let delay = Duration::from_secs(2);
    /// let event = RetryEvent::<i32>::new(1, 3, None, None, delay, Duration::from_millis(100));
    /// assert_eq!(event.next_delay(), delay);
    /// ```
    pub fn next_delay(&self) -> Duration {
        self.next_delay
    }

    /// Get total execution time
    ///
    /// # Returns
    ///
    /// Returns the total time elapsed from the first attempt until now
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let total = Duration::from_millis(500);
    /// let event = RetryEvent::<i32>::new(2, 3, None, None, Duration::from_secs(1), total);
    /// assert_eq!(event.total_duration(), total);
    /// ```
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }

    /// Check if there are remaining retry attempts
    ///
    /// # Returns
    ///
    /// Returns `true` if current attempt count is less than maximum attempts, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let event = RetryEvent::<i32>::new(2, 3, None, None, Duration::from_secs(1), Duration::from_millis(200));
    /// assert!(event.has_remaining_attempts());
    ///
    /// let event2 = RetryEvent::<i32>::new(3, 3, None, None, Duration::from_secs(1), Duration::from_millis(300));
    /// assert!(!event2.has_remaining_attempts());
    /// ```
    pub fn has_remaining_attempts(&self) -> bool {
        self.attempt_count < self.max_attempts
    }
}

/// Success event
///
/// Event triggered when an operation completes successfully, containing detailed success information.
///
/// # Features
///
/// - Stores the successful result
/// - Records total attempt count (including the final successful attempt)
/// - Tracks total time from start to success
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Use Cases
///
/// Used in success listeners for logging, sending success notifications, or collecting performance metrics.
///
/// # Example
///
/// ```rust
/// use prism3_retry::events::SuccessEvent;
/// use std::time::Duration;
///
/// let event = SuccessEvent::new(
///     String::from("Success result"),
///     2,
///     Duration::from_millis(300),
/// );
///
/// println!("Operation succeeded on attempt {}, took {:?}",
///     event.attempt_count(),
///     event.total_duration()
/// );
/// ```
///
/// # Author
///
/// Haixing Hu
#[derive(Debug)]
pub struct SuccessEvent<T> {
    /// Successful result
    result: T,
    /// Total attempt count
    attempt_count: u32,
    /// Total execution time
    total_duration: Duration,
}

impl<T> SuccessEvent<T> {
    /// Create success event
    ///
    /// # Parameters
    ///
    /// * `result` - Successful result
    /// * `attempt_count` - Total attempt count
    /// * `total_duration` - Total execution time
    ///
    /// # Returns
    ///
    /// Returns a newly created `SuccessEvent` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::SuccessEvent;
    /// use std::time::Duration;
    ///
    /// let event = SuccessEvent::new(42, 1, Duration::from_millis(100));
    /// assert_eq!(event.result(), &42);
    /// ```
    pub fn new(result: T, attempt_count: u32, total_duration: Duration) -> Self {
        Self {
            result,
            attempt_count,
            total_duration,
        }
    }

    /// Get successful result
    ///
    /// # Returns
    ///
    /// Returns reference to the result when the operation succeeded
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::SuccessEvent;
    /// use std::time::Duration;
    ///
    /// let event = SuccessEvent::new(String::from("Success"), 1, Duration::from_millis(100));
    /// assert_eq!(event.result(), "Success");
    /// ```
    pub fn result(&self) -> &T {
        &self.result
    }

    /// Get total attempt count
    ///
    /// # Returns
    ///
    /// Returns the total number of attempts from start to success
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::SuccessEvent;
    /// use std::time::Duration;
    ///
    /// let event = SuccessEvent::new(42, 3, Duration::from_millis(300));
    /// assert_eq!(event.attempt_count(), 3);
    /// ```
    pub fn attempt_count(&self) -> u32 {
        self.attempt_count
    }

    /// Get total execution time
    ///
    /// # Returns
    ///
    /// Returns the total time from the first attempt until success
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::SuccessEvent;
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_millis(500);
    /// let event = SuccessEvent::new(42, 2, duration);
    /// assert_eq!(event.total_duration(), duration);
    /// ```
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }
}

/// Failure event
///
/// Event triggered when an operation ultimately fails, indicating all retry attempts have been exhausted.
///
/// # Features
///
/// - Stores the error or result from the last failure
/// - Records total attempt count
/// - Tracks total time from start to failure
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Use Cases
///
/// Used in failure listeners for logging failures, sending alerts, or performing fault handling.
///
/// # Example
///
/// ```rust
/// use prism3_retry::events::FailureEvent;
/// use std::time::Duration;
/// use std::io::{Error, ErrorKind};
///
/// let error = Error::new(ErrorKind::TimedOut, "All retries timed out");
/// let event = FailureEvent::<String>::new(
///     Some(Box::new(error)),
///     None,
///     3,
///     Duration::from_secs(5),
/// );
///
/// println!("Operation failed after {} attempts, total time {:?}",
///     event.attempt_count(),
///     event.total_duration()
/// );
/// ```
///
/// # Author
///
/// Haixing Hu
#[derive(Debug)]
pub struct FailureEvent<T> {
    /// Last error
    last_error: Option<Box<dyn StdError + Send + Sync>>,
    /// Last result
    last_result: Option<T>,
    /// Total attempt count
    attempt_count: u32,
    /// Total execution time
    total_duration: Duration,
}

impl<T> FailureEvent<T> {
    /// Create failure event
    ///
    /// # Parameters
    ///
    /// * `last_error` - Error from the last failure (if any)
    /// * `last_result` - Result from the last failure (if any)
    /// * `attempt_count` - Total attempt count
    /// * `total_duration` - Total execution time
    ///
    /// # Returns
    ///
    /// Returns a newly created `FailureEvent` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::FailureEvent;
    /// use std::time::Duration;
    ///
    /// let event = FailureEvent::new(None, Some(String::from("Failed")), 3, Duration::from_secs(3));
    /// assert_eq!(event.attempt_count(), 3);
    /// ```
    pub fn new(
        last_error: Option<Box<dyn StdError + Send + Sync>>,
        last_result: Option<T>,
        attempt_count: u32,
        total_duration: Duration,
    ) -> Self {
        Self {
            last_error,
            last_result,
            attempt_count,
            total_duration,
        }
    }

    /// Get last error
    ///
    /// # Returns
    ///
    /// Returns reference to the error if last failure was due to error, None otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::FailureEvent;
    /// use std::time::Duration;
    /// use std::io::{Error, ErrorKind};
    ///
    /// let error = Error::new(ErrorKind::TimedOut, "Timeout");
    /// let event = FailureEvent::<i32>::new(Some(Box::new(error)), None, 3, Duration::from_secs(3));
    /// assert!(event.last_error().is_some());
    /// ```
    pub fn last_error(&self) -> Option<&(dyn StdError + Send + Sync)> {
        self.last_error.as_ref().map(|e| e.as_ref())
    }

    /// Get last result
    ///
    /// # Returns
    ///
    /// Returns reference to the result if last failure had a return value, None otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::FailureEvent;
    /// use std::time::Duration;
    ///
    /// let event = FailureEvent::new(None, Some(String::from("Failed")), 3, Duration::from_secs(3));
    /// assert_eq!(event.last_result(), Some(&String::from("Failed")));
    /// ```
    pub fn last_result(&self) -> Option<&T> {
        self.last_result.as_ref()
    }

    /// Get total attempt count
    ///
    /// # Returns
    ///
    /// Returns the total attempt count before failure
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::FailureEvent;
    /// use std::time::Duration;
    ///
    /// let event = FailureEvent::<i32>::new(None, None, 5, Duration::from_secs(5));
    /// assert_eq!(event.attempt_count(), 5);
    /// ```
    pub fn attempt_count(&self) -> u32 {
        self.attempt_count
    }

    /// Get total execution time
    ///
    /// # Returns
    ///
    /// Returns the total time from the first attempt until failure
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::FailureEvent;
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_secs(10);
    /// let event = FailureEvent::<i32>::new(None, None, 3, duration);
    /// assert_eq!(event.total_duration(), duration);
    /// ```
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }
}

/// Abort event
///
/// Event triggered when an operation is aborted, indicating the operation should not continue retrying.
///
/// # Features
///
/// - Stores the reason for abort (error or result)
/// - Records attempt count at time of abort
/// - Tracks total time from start to abort
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Use Cases
///
/// Used in abort listeners for logging abort reasons, sending alerts, or performing cleanup operations.
///
/// # Example
///
/// ```rust
/// use prism3_retry::events::{AbortEvent, AbortReason};
/// use std::time::Duration;
/// use std::io::{Error, ErrorKind};
///
/// let error = Error::new(ErrorKind::PermissionDenied, "Permission denied");
/// let reason: AbortReason<String> = AbortReason::Error(Box::new(error));
/// let event: AbortEvent<String> = AbortEvent::new(
///     reason,
///     2,
///     Duration::from_millis(500),
/// );
///
/// println!("Operation aborted on attempt {}",
///     event.attempt_count()
/// );
/// ```
///
/// # Author
///
/// Haixing Hu
#[derive(Debug)]
pub struct AbortEvent<T> {
    /// Abort reason
    reason: AbortReason<T>,
    /// Total attempt count
    attempt_count: u32,
    /// Total execution time
    total_duration: Duration,
}

impl<T> AbortEvent<T> {
    /// Create abort event
    ///
    /// # Parameters
    ///
    /// * `reason` - Reason for abort
    /// * `attempt_count` - Attempt count at time of abort
    /// * `total_duration` - Total execution time
    ///
    /// # Returns
    ///
    /// Returns a newly created `AbortEvent` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::{AbortEvent, AbortReason};
    /// use std::time::Duration;
    ///
    /// let reason = AbortReason::Result(String::from("INVALID"));
    /// let event = AbortEvent::new(reason, 1, Duration::from_millis(100));
    /// assert_eq!(event.attempt_count(), 1);
    /// ```
    pub fn new(reason: AbortReason<T>, attempt_count: u32, total_duration: Duration) -> Self {
        Self {
            reason,
            attempt_count,
            total_duration,
        }
    }

    /// Get abort reason
    ///
    /// # Returns
    ///
    /// Returns reference to the reason that caused the operation to abort
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::{AbortEvent, AbortReason};
    /// use std::time::Duration;
    ///
    /// let reason = AbortReason::Result(String::from("INVALID"));
    /// let event = AbortEvent::new(reason, 1, Duration::from_millis(100));
    /// match event.reason() {
    ///     AbortReason::Result(r) => assert_eq!(r, "INVALID"),
    ///     _ => panic!("Wrong reason type"),
    /// }
    /// ```
    pub fn reason(&self) -> &AbortReason<T> {
        &self.reason
    }

    /// Get total attempt count
    ///
    /// # Returns
    ///
    /// Returns the total attempt count at time of abort
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::{AbortEvent, AbortReason};
    /// use std::time::Duration;
    ///
    /// let reason = AbortReason::Result(42);
    /// let event = AbortEvent::new(reason, 2, Duration::from_millis(200));
    /// assert_eq!(event.attempt_count(), 2);
    /// ```
    pub fn attempt_count(&self) -> u32 {
        self.attempt_count
    }

    /// Get total execution time
    ///
    /// # Returns
    ///
    /// Returns the total time from the first attempt until abort
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::events::{AbortEvent, AbortReason};
    /// use std::time::Duration;
    ///
    /// let reason = AbortReason::Result(42);
    /// let duration = Duration::from_millis(300);
    /// let event = AbortEvent::new(reason, 1, duration);
    /// assert_eq!(event.total_duration(), duration);
    /// ```
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }
}

/// Retry event listener type
///
/// Callback function type for listening to retry events, called when an operation fails and is preparing to retry.
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Example
///
/// ```rust
/// use prism3_retry::events::{RetryEvent, RetryEventListener};
///
/// let listener: RetryEventListener<i32> = Box::new(|event| {
///     println!("Retry attempt {}, delay {:?}", event.attempt_count(), event.next_delay());
/// });
/// ```
pub type RetryEventListener<T> = Box<dyn Fn(RetryEvent<T>) + Send + Sync>;

/// Success event listener type
///
/// Callback function type for listening to success events, called when an operation completes successfully.
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Example
///
/// ```rust
/// use prism3_retry::events::{SuccessEvent, SuccessEventListener};
///
/// let listener: SuccessEventListener<i32> = Box::new(|event| {
///     println!("Operation succeeded, attempted {} times", event.attempt_count());
/// });
/// ```
pub type SuccessEventListener<T> = Box<dyn Fn(SuccessEvent<T>) + Send + Sync>;

/// Failure event listener type
///
/// Callback function type for listening to failure events, called when all retry attempts have failed.
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Example
///
/// ```rust
/// use prism3_retry::events::{FailureEvent, FailureEventListener};
///
/// let listener: FailureEventListener<i32> = Box::new(|event| {
///     println!("Operation failed, attempted {} times", event.attempt_count());
/// });
/// ```
pub type FailureEventListener<T> = Box<dyn Fn(FailureEvent<T>) + Send + Sync>;

/// Abort event listener type
///
/// Callback function type for listening to abort events, called when an operation is aborted.
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Example
///
/// ```rust
/// use prism3_retry::events::{AbortEvent, AbortEventListener};
///
/// let listener: AbortEventListener<i32> = Box::new(|event| {
///     println!("Operation aborted, attempted {} times", event.attempt_count());
/// });
/// ```
pub type AbortEventListener<T> = Box<dyn Fn(AbortEvent<T>) + Send + Sync>;
