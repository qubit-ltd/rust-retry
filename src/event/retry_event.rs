/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Retry Event
//!
//! Event triggered when an operation fails and is preparing to
//! retry.
//!
//! # Author
//!
//! Haixing Hu

use std::error::Error as StdError;
use std::time::Duration;

use prism3_function::readonly_consumer::BoxReadonlyConsumer;

/// Retry event
///
/// Event triggered when an operation fails and is preparing to
/// retry, containing detailed information about the retry.
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
/// Used in retry listeners for logging, sending monitoring data, or
/// implementing custom retry logic.
///
/// # Construction
///
/// This event can only be constructed using the builder pattern via
/// `RetryEventBuilder`. The builder pattern provides a fluent
/// interface for creating retry events with named parameters:
///
/// ```rust
/// use prism3_retry::event::retry_event::RetryEvent;
/// use std::time::Duration;
///
/// let event = RetryEvent::<String>::builder()
///     .attempt_count(1)
///     .max_attempts(3)
///     .last_result(Some(String::from("empty")))
///     .next_delay(Duration::from_secs(1))
///     .total_duration(Duration::from_millis(100))
///     .build();
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
    /// Create a builder for constructing `RetryEvent`
    ///
    /// # Returns
    ///
    /// Returns a new `RetryEventBuilder` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::retry_event::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let event = RetryEvent::<i32>::builder()
    ///     .attempt_count(1)
    ///     .max_attempts(3)
    ///     .next_delay(Duration::from_secs(1))
    ///     .total_duration(Duration::from_millis(100))
    ///     .build();
    /// ```
    pub fn builder() -> RetryEventBuilder<T> {
        RetryEventBuilder::new()
    }

    /// Get current attempt count
    ///
    /// # Returns
    ///
    /// Returns the current number of attempts already made (counting
    /// from 1)
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::retry_event::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let event = RetryEvent::<i32>::builder()
    ///     .attempt_count(2)
    ///     .max_attempts(3)
    ///     .next_delay(Duration::from_secs(1))
    ///     .total_duration(Duration::from_millis(200))
    ///     .build();
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
    /// use prism3_retry::event::retry_event::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let event = RetryEvent::<i32>::builder()
    ///     .attempt_count(1)
    ///     .max_attempts(5)
    ///     .next_delay(Duration::from_secs(1))
    ///     .total_duration(Duration::from_millis(100))
    ///     .build();
    /// assert_eq!(event.max_attempts(), 5);
    /// ```
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    /// Get error from last failure
    ///
    /// # Returns
    ///
    /// Returns reference to the error if last failure was due to
    /// error, None otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::retry_event::RetryEvent;
    /// use std::time::Duration;
    /// use std::io::{Error, ErrorKind};
    ///
    /// let error = Error::new(ErrorKind::TimedOut, "Timeout");
    /// let event = RetryEvent::<i32>::builder()
    ///     .attempt_count(1)
    ///     .max_attempts(3)
    ///     .last_error(Some(Box::new(error)))
    ///     .next_delay(Duration::from_secs(1))
    ///     .total_duration(Duration::from_millis(100))
    ///     .build();
    /// assert!(event.last_error().is_some());
    /// ```
    pub fn last_error(&self) -> Option<&(dyn StdError + Send + Sync)> {
        self.last_error.as_ref().map(|e| e.as_ref())
    }

    /// Get result from last failure
    ///
    /// # Returns
    ///
    /// Returns reference to the result if last failure had a return
    /// value, None otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::retry_event::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let event = RetryEvent::<String>::builder()
    ///     .last_result(Some(String::from("empty")))
    ///     .attempt_count(1)
    ///     .max_attempts(3)
    ///     .next_delay(Duration::from_secs(1))
    ///     .total_duration(Duration::from_millis(100))
    ///     .build();
    /// assert_eq!(
    ///     event.last_result(),
    ///     Some(&String::from("empty"))
    /// );
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
    /// use prism3_retry::event::retry_event::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let delay = Duration::from_secs(2);
    /// let event = RetryEvent::<i32>::builder()
    ///     .attempt_count(1)
    ///     .max_attempts(3)
    ///     .next_delay(delay)
    ///     .total_duration(Duration::from_millis(100))
    ///     .build();
    /// assert_eq!(event.next_delay(), delay);
    /// ```
    pub fn next_delay(&self) -> Duration {
        self.next_delay
    }

    /// Get total execution time
    ///
    /// # Returns
    ///
    /// Returns the total time elapsed from the first attempt until
    /// now
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::retry_event::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let total = Duration::from_millis(500);
    /// let event = RetryEvent::<i32>::builder()
    ///     .attempt_count(2)
    ///     .max_attempts(3)
    ///     .next_delay(Duration::from_secs(1))
    ///     .total_duration(total)
    ///     .build();
    /// assert_eq!(event.total_duration(), total);
    /// ```
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }

    /// Check if there are remaining retry attempts
    ///
    /// # Returns
    ///
    /// Returns `true` if current attempt count is less than maximum
    /// attempts, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::retry_event::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let event = RetryEvent::<i32>::builder()
    ///     .attempt_count(2)
    ///     .max_attempts(3)
    ///     .next_delay(Duration::from_secs(1))
    ///     .total_duration(Duration::from_millis(200))
    ///     .build();
    /// assert!(event.has_remaining_attempts());
    ///
    /// let event2 = RetryEvent::<i32>::builder()
    ///     .attempt_count(3)
    ///     .max_attempts(3)
    ///     .next_delay(Duration::from_secs(1))
    ///     .total_duration(Duration::from_millis(300))
    ///     .build();
    /// assert!(!event2.has_remaining_attempts());
    /// ```
    pub fn has_remaining_attempts(&self) -> bool {
        self.attempt_count < self.max_attempts
    }
}

/// Retry event listener type
///
/// Callback function type for listening to retry events, called when
/// an operation fails and is preparing to retry.
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
/// use prism3_retry::event::retry_event::{
///     RetryEvent,
///     RetryEventListener
/// };
/// use prism3_function::readonly_consumer::BoxReadonlyConsumer;
///
/// let listener: RetryEventListener<i32> =
///     BoxReadonlyConsumer::new(|event: &RetryEvent<i32>| {
///         println!(
///             "Retry attempt {}, delay {:?}",
///             event.attempt_count(),
///             event.next_delay()
///         );
///     });
/// ```
pub type RetryEventListener<T> = BoxReadonlyConsumer<RetryEvent<T>>;

/// Builder for constructing `RetryEvent`
///
/// Provides a fluent interface for building retry events with
/// optional fields. All fields have default values and can be set
/// independently.
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Example
///
/// ```rust
/// use prism3_retry::event::retry_event::RetryEvent;
/// use std::time::Duration;
///
/// let event = RetryEvent::<i32>::builder()
///     .attempt_count(2)
///     .max_attempts(5)
///     .next_delay(Duration::from_secs(2))
///     .total_duration(Duration::from_millis(500))
///     .build();
/// ```
///
/// # Author
///
/// Haixing Hu
#[derive(Debug)]
pub struct RetryEventBuilder<T> {
    attempt_count: u32,
    max_attempts: u32,
    last_error: Option<Box<dyn StdError + Send + Sync>>,
    last_result: Option<T>,
    next_delay: Duration,
    total_duration: Duration,
}

impl<T> RetryEventBuilder<T> {
    /// Create a new builder with default values
    ///
    /// # Returns
    ///
    /// Returns a new `RetryEventBuilder` instance with all fields
    /// set to their default values
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::retry_event::RetryEventBuilder;
    ///
    /// let builder = RetryEventBuilder::<i32>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            attempt_count: 0,
            max_attempts: 0,
            last_error: None,
            last_result: None,
            next_delay: Duration::default(),
            total_duration: Duration::default(),
        }
    }

    /// Set the current attempt count
    ///
    /// # Parameters
    ///
    /// * `attempt_count` - Current attempt count (counting from 1)
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::retry_event::RetryEvent;
    ///
    /// let builder = RetryEvent::<i32>::builder()
    ///     .attempt_count(3);
    /// ```
    pub fn attempt_count(mut self, attempt_count: u32) -> Self {
        self.attempt_count = attempt_count;
        self
    }

    /// Set the maximum attempt count
    ///
    /// # Parameters
    ///
    /// * `max_attempts` - Maximum number of attempts allowed
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::retry_event::RetryEvent;
    ///
    /// let builder = RetryEvent::<i32>::builder()
    ///     .max_attempts(5);
    /// ```
    pub fn max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
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
    /// use prism3_retry::event::retry_event::RetryEvent;
    /// use std::io::{Error, ErrorKind};
    ///
    /// let error = Error::new(ErrorKind::TimedOut, "Timeout");
    /// let builder = RetryEvent::<i32>::builder()
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
    /// use prism3_retry::event::retry_event::RetryEvent;
    ///
    /// let builder = RetryEvent::builder()
    ///     .last_result(Some(String::from("empty")));
    /// ```
    pub fn last_result(mut self, last_result: Option<T>) -> Self {
        self.last_result = last_result;
        self
    }

    /// Set the delay time for next retry
    ///
    /// # Parameters
    ///
    /// * `next_delay` - Duration to wait before the next retry
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::retry_event::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let builder = RetryEvent::<i32>::builder()
    ///     .next_delay(Duration::from_secs(2));
    /// ```
    pub fn next_delay(mut self, next_delay: Duration) -> Self {
        self.next_delay = next_delay;
        self
    }

    /// Set the total execution time
    ///
    /// # Parameters
    ///
    /// * `total_duration` - Total time elapsed from the first
    ///   attempt
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::retry_event::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let builder = RetryEvent::<i32>::builder()
    ///     .total_duration(Duration::from_millis(500));
    /// ```
    pub fn total_duration(mut self, total_duration: Duration) -> Self {
        self.total_duration = total_duration;
        self
    }

    /// Build the `RetryEvent`
    ///
    /// Consumes the builder and creates a new `RetryEvent` instance
    /// with the configured values.
    ///
    /// # Returns
    ///
    /// Returns a newly constructed `RetryEvent` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::retry_event::RetryEvent;
    /// use std::time::Duration;
    ///
    /// let event = RetryEvent::<i32>::builder()
    ///     .attempt_count(1)
    ///     .max_attempts(3)
    ///     .next_delay(Duration::from_secs(1))
    ///     .total_duration(Duration::from_millis(100))
    ///     .build();
    /// ```
    pub fn build(self) -> RetryEvent<T> {
        RetryEvent {
            attempt_count: self.attempt_count,
            max_attempts: self.max_attempts,
            last_error: self.last_error,
            last_result: self.last_result,
            next_delay: self.next_delay,
            total_duration: self.total_duration,
        }
    }
}

impl<T> Default for RetryEventBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}
