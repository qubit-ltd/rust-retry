/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Retry Error Types
//!
//! Defines error types used in the retry module.
//!
//! # Author
//!
//! Haixing Hu

use std::error::Error;
use std::fmt;

/// Error type for the retry module
///
/// Defines various error conditions that can occur during retry operations, including exceeding maximum retries,
/// exceeding duration limits, operation abortion, configuration errors, etc.
///
/// # Features
///
/// - Supports unified handling of multiple error types
/// - Provides detailed error information and context
/// - Supports error chain tracking (via the source method)
/// - Implements the standard Error trait for interoperability with other error types
///
/// # Use Cases
///
/// Suitable for operations requiring retry mechanisms, such as network requests, file operations, database connections, etc.
/// Returns a corresponding RetryError when retry strategies fail or encounter unrecoverable errors.
///
/// # Example
///
/// ```rust
/// use prism3_retry::RetryError;
///
/// // Create maximum attempts exceeded error
/// let error = RetryError::max_attempts_exceeded(5, 3);
/// println!("Error: {}", error);
///
/// // Create execution error
/// let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
/// let retry_error = RetryError::execution_error(io_error);
/// println!("Execution error: {}", retry_error);
/// ```
///
/// # Author
///
/// Haixing Hu
///
#[derive(Debug)]
pub enum RetryError {
    /// Maximum attempts exceeded
    ///
    /// Triggered when the number of retries reaches or exceeds the preset maximum retry count.
    ///
    /// # Fields
    ///
    /// * `attempts` - Actual number of attempts
    /// * `max_attempts` - Maximum allowed retry count
    MaxAttemptsExceeded { attempts: u32, max_attempts: u32 },

    /// Maximum duration exceeded
    ///
    /// Triggered when the total duration of retry operations exceeds the preset maximum duration.
    ///
    /// # Fields
    ///
    /// * `duration` - Actual time consumed
    /// * `max_duration` - Maximum allowed duration
    ///
    MaxDurationExceeded {
        duration: std::time::Duration,
        max_duration: std::time::Duration,
    },

    /// Single operation timeout
    ///
    /// Triggered when the execution time of a single operation exceeds the configured operation timeout.
    /// This differs from MaxDurationExceeded, which is for the total time limit of the entire retry process.
    ///
    /// # Fields
    ///
    /// * `duration` - Actual execution time
    /// * `timeout` - Configured timeout duration
    ///
    OperationTimeout {
        duration: std::time::Duration,
        timeout: std::time::Duration,
    },

    /// Operation aborted
    ///
    /// Triggered when retry operation is aborted by external factors (e.g., user cancellation, system signals).
    ///
    /// # Fields
    ///
    /// * `reason` - Description of the abort reason
    ///
    Aborted { reason: String },

    /// Configuration error
    ///
    /// Triggered when retry configuration parameters are invalid or conflicting.
    ///
    /// # Fields
    ///
    /// * `message` - Error description message
    ///
    ConfigError { message: String },

    /// Delay strategy error
    ///
    /// Triggered when delay strategy configuration is erroneous or an error occurs while calculating delays.
    ///
    /// # Fields
    ///
    /// * `message` - Error description message
    ///
    DelayStrategyError { message: String },

    /// Execution error
    ///
    /// Wraps the original error when the retried operation itself fails.
    ///
    /// # Fields
    ///
    /// * `source` - Original error, supports error chain tracking
    ///
    ExecutionError {
        source: Box<dyn Error + Send + Sync>,
    },

    /// Other errors
    ///
    /// Used to represent other error situations that don't fall into the above categories.
    ///
    /// # Fields
    ///
    /// * `message` - Error description message
    ///
    Other { message: String },
}

impl fmt::Display for RetryError {
    /// Format error information into a readable string
    ///
    /// Provides error descriptions for each error type, including relevant contextual information.
    ///
    /// # Parameters
    ///
    /// * `f` - Formatter
    ///
    /// # Returns
    ///
    /// Returns formatting result
    ///
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RetryError::MaxAttemptsExceeded {
                attempts,
                max_attempts,
            } => {
                write!(f, "Maximum attempts exceeded: {} (max: {})", attempts, max_attempts)
            }
            RetryError::MaxDurationExceeded {
                duration,
                max_duration,
            } => {
                write!(
                    f,
                    "Maximum duration exceeded: {:?} (max: {:?})",
                    duration, max_duration
                )
            }
            RetryError::OperationTimeout { duration, timeout } => {
                write!(
                    f,
                    "Operation timeout: execution time {:?} exceeded configured timeout {:?}",
                    duration, timeout
                )
            }
            RetryError::Aborted { reason } => {
                write!(f, "Operation aborted: {}", reason)
            }
            RetryError::ConfigError { message } => {
                write!(f, "Configuration error: {}", message)
            }
            RetryError::DelayStrategyError { message } => {
                write!(f, "Delay strategy error: {}", message)
            }
            RetryError::ExecutionError { source } => {
                write!(f, "Execution error: {}", source)
            }
            RetryError::Other { message } => {
                write!(f, "Other error: {}", message)
            }
        }
    }
}

impl Error for RetryError {
    /// Get the root cause of the error
    ///
    /// For ExecutionError type, returns the original error; for other types, returns None.
    /// This supports error chain tracking, aiding in debugging and error handling.
    ///
    /// # Returns
    ///
    /// Returns the root cause of the error, or None if it doesn't exist
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RetryError::ExecutionError { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl RetryError {
    /// Create maximum attempts exceeded error
    ///
    /// Use this method to create an error when the retry count reaches or exceeds the preset maximum retry count.
    ///
    /// # Parameters
    ///
    /// * `attempts` - Actual number of attempts
    /// * `max_attempts` - Maximum allowed retry count
    ///
    /// # Returns
    ///
    /// Returns a RetryError containing retry count information
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryError;
    ///
    /// let error = RetryError::max_attempts_exceeded(5, 3);
    /// assert!(error.to_string().contains("Maximum attempts exceeded"));
    /// ```
    pub fn max_attempts_exceeded(attempts: u32, max_attempts: u32) -> Self {
        RetryError::MaxAttemptsExceeded {
            attempts,
            max_attempts,
        }
    }

    /// Create maximum duration exceeded error
    ///
    /// Use this method to create an error when the total duration of retry operations exceeds the preset maximum duration.
    ///
    /// # Parameters
    ///
    /// * `duration` - Actual time consumed
    /// * `max_duration` - Maximum allowed duration
    ///
    /// # Returns
    ///
    /// Returns a RetryError containing time information
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryError;
    /// use std::time::Duration;
    ///
    /// let error = RetryError::max_duration_exceeded(
    ///     Duration::from_secs(10),
    ///     Duration::from_secs(5)
    /// );
    /// assert!(error.to_string().contains("Maximum duration exceeded"));
    /// ```
    pub fn max_duration_exceeded(
        duration: std::time::Duration,
        max_duration: std::time::Duration,
    ) -> Self {
        RetryError::MaxDurationExceeded {
            duration,
            max_duration,
        }
    }

    /// Create single operation timeout error
    ///
    /// Use this method to create an error when the execution time of a single operation exceeds the configured operation timeout.
    ///
    /// # Parameters
    ///
    /// * `duration` - Actual execution time
    /// * `timeout` - Configured timeout duration
    ///
    /// # Returns
    ///
    /// Returns a RetryError containing time information
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryError;
    /// use std::time::Duration;
    ///
    /// let error = RetryError::operation_timeout(
    ///     Duration::from_secs(10),
    ///     Duration::from_secs(5)
    /// );
    /// assert!(error.to_string().contains("Operation timeout"));
    /// ```
    pub fn operation_timeout(duration: std::time::Duration, timeout: std::time::Duration) -> Self {
        RetryError::OperationTimeout { duration, timeout }
    }

    /// Create abort error
    ///
    /// Use this method to create an error when the retry operation is aborted by external factors.
    ///
    /// # Parameters
    ///
    /// * `reason` - Reason for abortion
    ///
    /// # Returns
    ///
    /// Returns a RetryError containing the abort reason
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryError;
    ///
    /// let error = RetryError::aborted("User cancelled operation");
    /// assert!(error.to_string().contains("Operation aborted"));
    /// ```
    pub fn aborted(reason: &str) -> Self {
        RetryError::Aborted {
            reason: reason.to_string(),
        }
    }

    /// Create configuration error
    ///
    /// Use this method to create an error when retry configuration parameters are invalid or conflicting.
    ///
    /// # Parameters
    ///
    /// * `message` - Error description message
    ///
    /// # Returns
    ///
    /// Returns a RetryError containing error information
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryError;
    ///
    /// let error = RetryError::config_error("Maximum retry count cannot be negative");
    /// assert!(error.to_string().contains("Configuration error"));
    /// ```
    pub fn config_error(message: &str) -> Self {
        RetryError::ConfigError {
            message: message.to_string(),
        }
    }

    /// Create delay strategy error
    ///
    /// Use this method to create an error when delay strategy configuration is erroneous or an error occurs while calculating delays.
    ///
    /// # Parameters
    ///
    /// * `message` - Error description message
    ///
    /// # Returns
    ///
    /// Returns a RetryError containing error information
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryError;
    ///
    /// let error = RetryError::delay_strategy_error("Delay time calculation overflow");
    /// assert!(error.to_string().contains("Delay strategy error"));
    /// ```
    pub fn delay_strategy_error(message: &str) -> Self {
        RetryError::DelayStrategyError {
            message: message.to_string(),
        }
    }

    /// Create execution error
    ///
    /// Use this method to wrap the original error as RetryError when the retried operation itself fails.
    ///
    /// # Parameters
    ///
    /// * `error` - Original error, must implement Error + Send + Sync trait
    ///
    /// # Returns
    ///
    /// Returns a RetryError wrapping the original error
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryError;
    ///
    /// let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    /// let retry_error = RetryError::execution_error(io_error);
    /// assert!(retry_error.to_string().contains("Execution error"));
    /// ```
    pub fn execution_error<E: Error + Send + Sync + 'static>(error: E) -> Self {
        RetryError::ExecutionError {
            source: Box::new(error),
        }
    }

    /// Create execution error (from Box<dyn Error>)
    ///
    /// When you already have a Box<dyn Error>, use this method to create an execution error directly.
    /// This avoids additional boxing operations.
    ///
    /// # Parameters
    ///
    /// * `error` - Boxed error
    ///
    /// # Returns
    ///
    /// Returns a RetryError wrapping the original error
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryError;
    ///
    /// let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    /// let boxed_error = Box::new(io_error);
    /// let retry_error = RetryError::execution_error_box(boxed_error);
    /// assert!(retry_error.to_string().contains("Execution error"));
    /// ```
    pub fn execution_error_box(error: Box<dyn Error + Send + Sync>) -> Self {
        RetryError::ExecutionError { source: error }
    }

    /// Create other error
    ///
    /// Use this method to create an error for other error situations that don't fall into the above categories.
    ///
    /// # Parameters
    ///
    /// * `message` - Error description message
    ///
    /// # Returns
    ///
    /// Returns a RetryError containing error information
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryError;
    ///
    /// let error = RetryError::other("Unknown error type");
    /// assert!(error.to_string().contains("Other error"));
    /// ```
    pub fn other(message: &str) -> Self {
        RetryError::Other {
            message: message.to_string(),
        }
    }
}

/// Retry result type alias
///
/// Represents the result of a retry operation, returning type T on success and RetryError on failure.
/// This is the unified return type for all operations in the retry module.
///
/// # Type Parameters
///
/// * `T` - The data type returned on success
///
/// # Example
///
/// ```rust
/// use prism3_retry::{RetryResult, RetryError};
///
/// fn retry_operation() -> RetryResult<String> {
///     // Simulate retry operation
///     Ok("Operation successful".to_string())
/// }
///
/// fn retry_operation_failed() -> RetryResult<String> {
///     Err(RetryError::other("Operation failed"))
/// }
/// ```
pub type RetryResult<T> = Result<T, RetryError>;

/// Convert from standard error types
///
/// Provides automatic conversion from std::io::Error to RetryError.
/// This simplifies error handling, allowing direct use of the ? operator.
///
/// # Parameters
///
/// * `error` - IO error
///
/// # Returns
///
/// Returns a RetryError wrapping the IO error
///
/// # Example
///
/// ```rust
/// use prism3_retry::{RetryError, RetryResult};
///
/// fn io_operation() -> RetryResult<()> {
///     let file = std::fs::File::open("nonexistent_file.txt")?;
///     // Do something with file
///     Ok(())
/// }
/// ```
impl From<std::io::Error> for RetryError {
    /// Convert std::io::Error to RetryError
    fn from(error: std::io::Error) -> Self {
        RetryError::ExecutionError {
            source: Box::new(error),
        }
    }
}

/// Convert from boxed error types
///
/// Provides automatic conversion from Box<dyn Error + Send + Sync> to RetryError.
/// This allows converting any boxed error directly to RetryError.
///
/// # Parameters
///
/// * `error` - Boxed error
///
/// # Returns
///
/// Returns a RetryError wrapping the original error
///
/// # Example
///
/// ```rust
/// use prism3_retry::RetryError;
///
/// let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
/// let boxed_error: Box<dyn std::error::Error + Send + Sync> = Box::new(io_error);
/// let retry_error: RetryError = boxed_error.into();
/// ```
impl From<Box<dyn Error + Send + Sync>> for RetryError {
    /// Convert boxed error to RetryError
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        RetryError::ExecutionError { source: error }
    }
}
