/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Failure Event
//!
//! Event triggered when an operation ultimately fails.
//!
//! # Author
//!
//! Haixing Hu

use std::error::Error as StdError;
use std::time::Duration;

use prism3_function::readonly_consumer::BoxReadonlyConsumer;

/// Failure event
///
/// Event triggered when an operation ultimately fails, indicating
/// all retry attempts have been exhausted.
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
/// Used in failure listeners for logging failures, sending alerts,
/// or performing fault handling.
///
/// # Construction
///
/// This event must be constructed using the builder pattern via
/// `FailureEventBuilder`. The builder pattern provides a more
/// readable and flexible way to create failure events with named
/// parameters:
///
/// ```rust
/// use prism3_retry::event::failure_event::FailureEvent;
/// use std::time::Duration;
/// use std::io::{Error, ErrorKind};
///
/// let error = Error::new(
///     ErrorKind::TimedOut,
///     "All retries timed out"
/// );
///
/// // Using builder pattern - recommended approach
/// let event = FailureEvent::<String>::builder()
///     .last_error(Some(Box::new(error)))
///     .attempt_count(3)
///     .total_duration(Duration::from_secs(5))
///     .build();
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
    /// Create a builder for constructing `FailureEvent`
    ///
    /// # Returns
    ///
    /// Returns a new `FailureEventBuilder` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::failure_event::FailureEvent;
    /// use std::time::Duration;
    ///
    /// let event = FailureEvent::<String>::builder()
    ///     .last_result(Some(String::from("Failed")))
    ///     .attempt_count(3)
    ///     .total_duration(Duration::from_secs(3))
    ///     .build();
    /// assert_eq!(event.attempt_count(), 3);
    /// ```
    pub fn builder() -> FailureEventBuilder<T> {
        FailureEventBuilder::new()
    }

    /// Get last error
    ///
    /// # Returns
    ///
    /// Returns reference to the error if last failure was due to
    /// error, None otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::failure_event::FailureEvent;
    /// use std::time::Duration;
    /// use std::io::{Error, ErrorKind};
    ///
    /// let error = Error::new(ErrorKind::TimedOut, "Timeout");
    /// let event = FailureEvent::<i32>::builder()
    ///     .last_error(Some(Box::new(error)))
    ///     .attempt_count(3)
    ///     .total_duration(Duration::from_secs(3))
    ///     .build();
    /// assert!(event.last_error().is_some());
    /// ```
    pub fn last_error(&self) -> Option<&(dyn StdError + Send + Sync)> {
        self.last_error.as_ref().map(|e| e.as_ref())
    }

    /// Get last result
    ///
    /// # Returns
    ///
    /// Returns reference to the result if last failure had a return
    /// value, None otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::failure_event::FailureEvent;
    /// use std::time::Duration;
    ///
    /// let event = FailureEvent::builder()
    ///     .last_result(Some(String::from("Failed")))
    ///     .attempt_count(3)
    ///     .total_duration(Duration::from_secs(3))
    ///     .build();
    /// assert_eq!(
    ///     event.last_result(),
    ///     Some(&String::from("Failed"))
    /// );
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
    /// use prism3_retry::event::failure_event::FailureEvent;
    /// use std::time::Duration;
    ///
    /// let event = FailureEvent::<i32>::builder()
    ///     .attempt_count(5)
    ///     .total_duration(Duration::from_secs(5))
    ///     .build();
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
    /// use prism3_retry::event::failure_event::FailureEvent;
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_secs(10);
    /// let event = FailureEvent::<i32>::builder()
    ///     .attempt_count(3)
    ///     .total_duration(duration)
    ///     .build();
    /// assert_eq!(event.total_duration(), duration);
    /// ```
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }
}

/// Failure event listener type
///
/// Callback function type for listening to failure events, called
/// when all retry attempts have failed.
///
/// Uses `BoxReadonlyConsumer` from `prism3-function` to provide
/// readonly event consumption with single ownership.
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Example
///
/// ```rust
/// use prism3_retry::event::failure_event::{
///     FailureEvent,
///     FailureEventListener
/// };
/// use prism3_function::readonly_consumer::BoxReadonlyConsumer;
///
/// let listener: FailureEventListener<i32> =
///     BoxReadonlyConsumer::new(|event: &FailureEvent<i32>| {
///         println!(
///             "Operation failed, attempted {} times",
///             event.attempt_count()
///         );
///     });
/// ```
pub type FailureEventListener<T> = BoxReadonlyConsumer<FailureEvent<T>>;

/// Builder for constructing `FailureEvent`
///
/// Provides a fluent interface for building failure events with
/// optional error and result fields.
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Example
///
/// ```rust
/// use prism3_retry::event::failure_event::FailureEvent;
/// use std::time::Duration;
///
/// let event = FailureEvent::<String>::builder()
///     .attempt_count(3)
///     .total_duration(Duration::from_secs(5))
///     .build();
/// ```
///
/// # Author
///
/// Haixing Hu
#[derive(Debug)]
#[allow(clippy::new_without_default)]
pub struct FailureEventBuilder<T> {
    last_error: Option<Box<dyn StdError + Send + Sync>>,
    last_result: Option<T>,
    attempt_count: u32,
    total_duration: Duration,
}

impl<T> FailureEventBuilder<T> {
    /// Create a new builder with default values
    ///
    /// # Returns
    ///
    /// Returns a new `FailureEventBuilder` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::failure_event::FailureEventBuilder;
    ///
    /// let builder = FailureEventBuilder::<i32>::new();
    /// ```
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            last_error: None,
            last_result: None,
            attempt_count: 0,
            total_duration: Duration::default(),
        }
    }

    /// Set the error from last failure
    ///
    /// # Parameters
    ///
    /// * `last_error` - Optional error from the last failure
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::failure_event::FailureEvent;
    /// use std::io::{Error, ErrorKind};
    ///
    /// let error = Error::new(ErrorKind::TimedOut, "Timeout");
    /// let builder = FailureEvent::<i32>::builder()
    ///     .last_error(Some(Box::new(error)));
    /// ```
    pub fn last_error(mut self, last_error: Option<Box<dyn StdError + Send + Sync>>) -> Self {
        self.last_error = last_error;
        self
    }

    /// Set the result from last failure
    ///
    /// # Parameters
    ///
    /// * `last_result` - Optional result from the last failure
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::failure_event::FailureEvent;
    ///
    /// let builder = FailureEvent::builder()
    ///     .last_result(Some(String::from("Failed")));
    /// ```
    pub fn last_result(mut self, last_result: Option<T>) -> Self {
        self.last_result = last_result;
        self
    }

    /// Set the total attempt count
    ///
    /// # Parameters
    ///
    /// * `attempt_count` - Total number of attempts before failure
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::failure_event::FailureEvent;
    ///
    /// let builder = FailureEvent::<i32>::builder()
    ///     .attempt_count(5);
    /// ```
    pub fn attempt_count(mut self, attempt_count: u32) -> Self {
        self.attempt_count = attempt_count;
        self
    }

    /// Set the total execution time
    ///
    /// # Parameters
    ///
    /// * `total_duration` - Total time from start to failure
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::failure_event::FailureEvent;
    /// use std::time::Duration;
    ///
    /// let builder = FailureEvent::<i32>::builder()
    ///     .total_duration(Duration::from_secs(10));
    /// ```
    pub fn total_duration(mut self, total_duration: Duration) -> Self {
        self.total_duration = total_duration;
        self
    }

    /// Build the `FailureEvent`
    ///
    /// Consumes the builder and creates a new `FailureEvent`
    /// instance with the configured values.
    ///
    /// # Returns
    ///
    /// Returns a newly constructed `FailureEvent` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::failure_event::FailureEvent;
    /// use std::time::Duration;
    ///
    /// let event = FailureEvent::<String>::builder()
    ///     .attempt_count(3)
    ///     .total_duration(Duration::from_secs(5))
    ///     .build();
    /// ```
    pub fn build(self) -> FailureEvent<T> {
        FailureEvent {
            last_error: self.last_error,
            last_result: self.last_result,
            attempt_count: self.attempt_count,
            total_duration: self.total_duration,
        }
    }
}
