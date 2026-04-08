/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Success Event
//!
//! Event triggered when an operation completes successfully.
//!
//! # Author
//!
//! Haixing Hu

use std::time::Duration;

use qubit_function::BoxConsumer;

/// Success event
///
/// Event triggered when an operation completes successfully,
/// containing detailed success information.
///
/// # Features
///
/// - Stores the successful result
/// - Records total attempt count (including the final successful
///   attempt)
/// - Tracks total time from start to success
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Use Cases
///
/// Used in success listeners for logging, sending success
/// notifications, or collecting performance metrics.
///
/// # Construction
///
/// This event can only be constructed using the builder pattern via
/// `SuccessEventBuilder`. The builder pattern provides more readable
/// code with named parameters:
///
/// ```rust
/// use qubit_retry::event::success_event::SuccessEvent;
/// use std::time::Duration;
///
/// // Using builder pattern - more readable
/// let event = SuccessEvent::builder()
///     .result(String::from("Success result"))
///     .attempt_count(2)
///     .total_duration(Duration::from_millis(300))
///     .build();
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
    /// Create a builder for constructing `SuccessEvent`
    ///
    /// # Returns
    ///
    /// Returns a new `SuccessEventBuilder` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_retry::event::success_event::SuccessEvent;
    /// use std::time::Duration;
    ///
    /// let event = SuccessEvent::builder()
    ///     .result(42)
    ///     .attempt_count(1)
    ///     .total_duration(Duration::from_millis(100))
    ///     .build();
    /// assert_eq!(event.result(), &42);
    /// ```
    pub fn builder() -> SuccessEventBuilder<T> {
        SuccessEventBuilder::new()
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
    /// use qubit_retry::event::success_event::SuccessEvent;
    /// use std::time::Duration;
    ///
    /// let event = SuccessEvent::builder()
    ///     .result(String::from("Success"))
    ///     .attempt_count(1)
    ///     .total_duration(Duration::from_millis(100))
    ///     .build();
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
    /// use qubit_retry::event::success_event::SuccessEvent;
    /// use std::time::Duration;
    ///
    /// let event = SuccessEvent::builder()
    ///     .result(42)
    ///     .attempt_count(3)
    ///     .total_duration(Duration::from_millis(300))
    ///     .build();
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
    /// use qubit_retry::event::success_event::SuccessEvent;
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_millis(500);
    /// let event = SuccessEvent::builder()
    ///     .result(42)
    ///     .attempt_count(2)
    ///     .total_duration(duration)
    ///     .build();
    /// assert_eq!(event.total_duration(), duration);
    /// ```
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }
}

/// Success event listener type
///
/// Callback function type for listening to success events, called
/// when an operation completes successfully.
///
/// Uses `BoxConsumer` from `qubit-function` to provide
/// readonly event consumption with single ownership.
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Example
///
/// ```rust
/// use qubit_retry::event::success_event::{
///     SuccessEvent,
///     SuccessEventListener
/// };
/// use qubit_function::BoxConsumer;
///
/// let listener: SuccessEventListener<i32> =
///     BoxConsumer::new(|event: &SuccessEvent<i32>| {
///         println!(
///             "Operation succeeded, attempted {} times",
///             event.attempt_count()
///         );
///     });
/// ```
pub type SuccessEventListener<T> = BoxConsumer<SuccessEvent<T>>;

/// Builder for constructing `SuccessEvent`
///
/// Provides a fluent interface for building success events. The
/// result field must be set before building.
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Example
///
/// ```rust
/// use qubit_retry::event::success_event::SuccessEvent;
/// use std::time::Duration;
///
/// let event = SuccessEvent::builder()
///     .result(42)
///     .attempt_count(2)
///     .total_duration(Duration::from_millis(300))
///     .build();
/// ```
///
/// # Author
///
/// Haixing Hu
#[derive(Debug)]
#[allow(clippy::new_without_default)]
pub struct SuccessEventBuilder<T> {
    result: Option<T>,
    attempt_count: u32,
    total_duration: Duration,
}

impl<T> SuccessEventBuilder<T> {
    /// Create a new builder with default values
    ///
    /// # Returns
    ///
    /// Returns a new `SuccessEventBuilder` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_retry::event::success_event::SuccessEventBuilder;
    ///
    /// let builder = SuccessEventBuilder::<i32>::new();
    /// ```
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            result: None,
            attempt_count: 0,
            total_duration: Duration::default(),
        }
    }

    /// Set the successful result
    ///
    /// # Parameters
    ///
    /// * `result` - The successful result value
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_retry::event::success_event::SuccessEvent;
    ///
    /// let builder = SuccessEvent::builder()
    ///     .result(String::from("Success"));
    /// ```
    pub fn result(mut self, result: T) -> Self {
        self.result = Some(result);
        self
    }

    /// Set the total attempt count
    ///
    /// # Parameters
    ///
    /// * `attempt_count` - Total number of attempts until success
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_retry::event::success_event::SuccessEvent;
    ///
    /// let builder = SuccessEvent::<i32>::builder()
    ///     .attempt_count(3);
    /// ```
    pub fn attempt_count(mut self, attempt_count: u32) -> Self {
        self.attempt_count = attempt_count;
        self
    }

    /// Set the total execution time
    ///
    /// # Parameters
    ///
    /// * `total_duration` - Total time from start to success
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_retry::event::success_event::SuccessEvent;
    /// use std::time::Duration;
    ///
    /// let builder = SuccessEvent::<i32>::builder()
    ///     .total_duration(Duration::from_millis(500));
    /// ```
    pub fn total_duration(mut self, total_duration: Duration) -> Self {
        self.total_duration = total_duration;
        self
    }

    /// Build the `SuccessEvent`
    ///
    /// Consumes the builder and creates a new `SuccessEvent`
    /// instance.
    ///
    /// # Panics
    ///
    /// Panics if the result field has not been set.
    ///
    /// # Returns
    ///
    /// Returns a newly constructed `SuccessEvent` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_retry::event::success_event::SuccessEvent;
    /// use std::time::Duration;
    ///
    /// let event = SuccessEvent::builder()
    ///     .result(42)
    ///     .attempt_count(1)
    ///     .total_duration(Duration::from_millis(100))
    ///     .build();
    /// ```
    pub fn build(self) -> SuccessEvent<T> {
        SuccessEvent {
            result: self.result.expect("result must be set"),
            attempt_count: self.attempt_count,
            total_duration: self.total_duration,
        }
    }
}
