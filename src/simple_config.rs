/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Simple Retry Configuration Implementation
//!
//! Provides the simplest retry configuration implementation, with all fields stored directly in the struct.
//!
//! # Author
//!
//! Haixing Hu

use super::config::RetryConfig;
use super::delay_strategy::RetryDelayStrategy;
use std::time::Duration;

/// Simple retry configuration implementation
///
/// This is the simplest `RetryConfig` implementation, with all configuration fields stored directly in the struct,
/// without depending on an external configuration system. Suitable for scenarios requiring direct control over all configurations.
///
/// # Features
///
/// - **Simple and Direct**: All fields stored directly, no configuration system needed
/// - **Type Safe**: Use Rust's type system to ensure correct configuration
/// - **High Performance**: No configuration queries, direct field access
/// - **Easy to Understand**: Clear structure, simple code
///
/// # Example
///
/// ```rust
/// use qubit_retry::{SimpleRetryConfig, RetryConfig, RetryDelayStrategy};
/// use std::time::Duration;
///
/// let mut config = SimpleRetryConfig::new();
/// config
///     .set_max_attempts(3)
///     .set_max_duration(Some(Duration::from_secs(30)))
///     .set_fixed_delay_strategy(Duration::from_secs(1));
///
/// assert_eq!(config.max_attempts(), 3);
/// assert_eq!(config.max_duration(), Some(Duration::from_secs(30)));
/// ```
///
/// # Author
///
/// Haixing Hu
#[derive(Debug, Clone)]
pub struct SimpleRetryConfig {
    /// Maximum number of attempts
    max_attempts: u32,
    /// Delay strategy
    delay_strategy: RetryDelayStrategy,
    /// Jitter factor
    jitter_factor: f64,
    /// Maximum duration
    max_duration: Option<Duration>,
    /// Single operation timeout
    operation_timeout: Option<Duration>,
}

impl SimpleRetryConfig {
    /// Create a new `SimpleRetryConfig` instance with default configuration
    ///
    /// # Returns
    ///
    /// Returns a new `SimpleRetryConfig` instance with default configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_retry::{SimpleRetryConfig, RetryConfig};
    ///
    /// let config = SimpleRetryConfig::new();
    /// assert_eq!(config.max_attempts(), 5);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            max_attempts: Self::DEFAULT_MAX_ATTEMPTS,
            delay_strategy: Self::DEFAULT_DELAY_STRATEGY,
            jitter_factor: Self::DEFAULT_JITTER_FACTOR,
            max_duration: None,
            operation_timeout: None,
        }
    }

    /// Create a new `SimpleRetryConfig` instance with specified parameters
    ///
    /// # Parameters
    ///
    /// * `max_attempts` - Maximum number of attempts
    /// * `delay_strategy` - Delay strategy
    /// * `jitter_factor` - Jitter factor
    /// * `max_duration` - Maximum duration
    /// * `operation_timeout` - Single operation timeout
    ///
    /// # Returns
    ///
    /// Returns a new `SimpleRetryConfig` instance with the specified parameters
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_retry::{SimpleRetryConfig, RetryDelayStrategy};
    /// use std::time::Duration;
    ///
    /// let config = SimpleRetryConfig::with_params(
    ///     3,
    ///     RetryDelayStrategy::fixed(Duration::from_secs(1)),
    ///     0.1,
    ///     Some(Duration::from_secs(30)),
    ///     Some(Duration::from_secs(5)),
    /// );
    /// ```
    #[inline]
    pub fn with_params(
        max_attempts: u32,
        delay_strategy: RetryDelayStrategy,
        jitter_factor: f64,
        max_duration: Option<Duration>,
        operation_timeout: Option<Duration>,
    ) -> Self {
        Self {
            max_attempts,
            delay_strategy,
            jitter_factor,
            max_duration,
            operation_timeout,
        }
    }
}

impl Default for SimpleRetryConfig {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl RetryConfig for SimpleRetryConfig {
    #[inline]
    fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    #[inline]
    fn set_max_attempts(&mut self, max_attempts: u32) -> &mut Self {
        self.max_attempts = max_attempts;
        self
    }

    #[inline]
    fn max_duration(&self) -> Option<Duration> {
        self.max_duration
    }

    #[inline]
    fn set_max_duration(&mut self, max_duration: Option<Duration>) -> &mut Self {
        self.max_duration = max_duration;
        self
    }

    #[inline]
    fn operation_timeout(&self) -> Option<Duration> {
        self.operation_timeout
    }

    #[inline]
    fn set_operation_timeout(&mut self, timeout: Option<Duration>) -> &mut Self {
        self.operation_timeout = timeout;
        self
    }

    #[inline]
    fn delay_strategy(&self) -> RetryDelayStrategy {
        self.delay_strategy.clone()
    }

    #[inline]
    fn set_delay_strategy(&mut self, delay_strategy: RetryDelayStrategy) -> &mut Self {
        self.delay_strategy = delay_strategy;
        self
    }

    #[inline]
    fn jitter_factor(&self) -> f64 {
        self.jitter_factor
    }

    #[inline]
    fn set_jitter_factor(&mut self, jitter_factor: f64) -> &mut Self {
        self.jitter_factor = jitter_factor;
        self
    }
}
