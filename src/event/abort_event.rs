/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Abort Event
//!
//! Event triggered when an operation is aborted.
//!
//! # Author
//!
//! Haixing Hu

use std::time::Duration;

use prism3_function::readonly_consumer::BoxReadonlyConsumer;

use super::abort_reason::AbortReason;

/// Abort event
///
/// Event triggered when an operation is aborted, indicating the
/// operation should not continue retrying.
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
/// Used in abort listeners for logging abort reasons, sending
/// alerts, or performing cleanup operations.
///
/// # Construction
///
/// This event can only be constructed using the builder pattern via
/// `AbortEventBuilder`. The builder pattern provides more readable code
/// with named parameters:
///
/// ```rust
/// use prism3_retry::event::abort_event::AbortEvent;
/// use prism3_retry::event::abort_reason::AbortReason;
/// use std::time::Duration;
/// use std::io::{Error, ErrorKind};
///
/// let error = Error::new(
///     ErrorKind::PermissionDenied,
///     "Permission denied"
/// );
/// let reason: AbortReason<String> = AbortReason::Error(
///     Box::new(error)
/// );
///
/// // Using builder pattern - more readable
/// let event: AbortEvent<String> = AbortEvent::builder()
///     .reason(reason)
///     .attempt_count(2)
///     .total_duration(Duration::from_millis(500))
///     .build();
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
    /// Create a builder for constructing `AbortEvent`
    ///
    /// # Returns
    ///
    /// Returns a new `AbortEventBuilder` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::abort_event::AbortEvent;
    /// use prism3_retry::event::abort_reason::AbortReason;
    /// use std::time::Duration;
    ///
    /// let reason = AbortReason::Result(String::from("INVALID"));
    /// let event = AbortEvent::builder()
    ///     .reason(reason)
    ///     .attempt_count(1)
    ///     .total_duration(Duration::from_millis(100))
    ///     .build();
    /// assert_eq!(event.attempt_count(), 1);
    /// ```
    pub fn builder() -> AbortEventBuilder<T> {
        AbortEventBuilder::new()
    }

    /// Get abort reason
    ///
    /// # Returns
    ///
    /// Returns reference to the reason that caused the operation to
    /// abort
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::abort_event::AbortEvent;
    /// use prism3_retry::event::abort_reason::AbortReason;
    /// use std::time::Duration;
    ///
    /// let reason = AbortReason::Result(String::from("INVALID"));
    /// let event = AbortEvent::builder()
    ///     .reason(reason)
    ///     .attempt_count(1)
    ///     .total_duration(Duration::from_millis(100))
    ///     .build();
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
    /// use prism3_retry::event::abort_event::AbortEvent;
    /// use prism3_retry::event::abort_reason::AbortReason;
    /// use std::time::Duration;
    ///
    /// let reason = AbortReason::Result(42);
    /// let event = AbortEvent::builder()
    ///     .reason(reason)
    ///     .attempt_count(2)
    ///     .total_duration(Duration::from_millis(200))
    ///     .build();
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
    /// use prism3_retry::event::abort_event::AbortEvent;
    /// use prism3_retry::event::abort_reason::AbortReason;
    /// use std::time::Duration;
    ///
    /// let reason = AbortReason::Result(42);
    /// let duration = Duration::from_millis(300);
    /// let event = AbortEvent::builder()
    ///     .reason(reason)
    ///     .attempt_count(1)
    ///     .total_duration(duration)
    ///     .build();
    /// assert_eq!(event.total_duration(), duration);
    /// ```
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }
}

/// Abort event listener type
///
/// Callback function type for listening to abort events, called when
/// an operation is aborted.
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
/// use prism3_retry::event::abort_event::{
///     AbortEvent,
///     AbortEventListener
/// };
/// use prism3_function::readonly_consumer::BoxReadonlyConsumer;
///
/// let listener: AbortEventListener<i32> =
///     BoxReadonlyConsumer::new(|event: &AbortEvent<i32>| {
///         println!(
///             "Operation aborted, attempted {} times",
///             event.attempt_count()
///         );
///     });
/// ```
pub type AbortEventListener<T> = BoxReadonlyConsumer<AbortEvent<T>>;

/// Builder for constructing `AbortEvent`
///
/// Provides a fluent interface for building abort events. The reason
/// field must be set before building.
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
///
/// # Example
///
/// ```rust
/// use prism3_retry::event::abort_event::AbortEvent;
/// use prism3_retry::event::abort_reason::AbortReason;
/// use std::time::Duration;
///
/// let reason = AbortReason::Result(String::from("INVALID"));
/// let event = AbortEvent::builder()
///     .reason(reason)
///     .attempt_count(2)
///     .total_duration(Duration::from_millis(500))
///     .build();
/// ```
///
/// # Author
///
/// Haixing Hu
#[derive(Debug)]
pub struct AbortEventBuilder<T> {
    reason: Option<AbortReason<T>>,
    attempt_count: u32,
    total_duration: Duration,
}

impl<T> AbortEventBuilder<T> {
    /// Create a new builder with default values
    ///
    /// # Returns
    ///
    /// Returns a new `AbortEventBuilder` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::abort_event::AbortEventBuilder;
    ///
    /// let builder = AbortEventBuilder::<i32>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            reason: None,
            attempt_count: 0,
            total_duration: Duration::default(),
        }
    }

    /// Set the abort reason
    ///
    /// # Parameters
    ///
    /// * `reason` - The reason for aborting the operation
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::abort_event::AbortEvent;
    /// use prism3_retry::event::abort_reason::AbortReason;
    ///
    /// let reason = AbortReason::Result(String::from("INVALID"));
    /// let builder = AbortEvent::builder().reason(reason);
    /// ```
    pub fn reason(mut self, reason: AbortReason<T>) -> Self {
        self.reason = Some(reason);
        self
    }

    /// Set the attempt count at time of abort
    ///
    /// # Parameters
    ///
    /// * `attempt_count` - Number of attempts made before abort
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::abort_event::AbortEvent;
    ///
    /// let builder = AbortEvent::<i32>::builder()
    ///     .attempt_count(2);
    /// ```
    pub fn attempt_count(mut self, attempt_count: u32) -> Self {
        self.attempt_count = attempt_count;
        self
    }

    /// Set the total execution time
    ///
    /// # Parameters
    ///
    /// * `total_duration` - Total time from start to abort
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::abort_event::AbortEvent;
    /// use std::time::Duration;
    ///
    /// let builder = AbortEvent::<i32>::builder()
    ///     .total_duration(Duration::from_millis(500));
    /// ```
    pub fn total_duration(mut self, total_duration: Duration) -> Self {
        self.total_duration = total_duration;
        self
    }

    /// Build the `AbortEvent`
    ///
    /// Consumes the builder and creates a new `AbortEvent` instance.
    ///
    /// # Panics
    ///
    /// Panics if the reason field has not been set.
    ///
    /// # Returns
    ///
    /// Returns a newly constructed `AbortEvent` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::event::abort_event::AbortEvent;
    /// use prism3_retry::event::abort_reason::AbortReason;
    /// use std::time::Duration;
    ///
    /// let reason = AbortReason::Result(42);
    /// let event = AbortEvent::builder()
    ///     .reason(reason)
    ///     .attempt_count(1)
    ///     .total_duration(Duration::from_millis(100))
    ///     .build();
    /// ```
    pub fn build(self) -> AbortEvent<T> {
        AbortEvent {
            reason: self.reason.expect("reason must be set"),
            attempt_count: self.attempt_count,
            total_duration: self.total_duration,
        }
    }
}

impl<T> Default for AbortEventBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}
