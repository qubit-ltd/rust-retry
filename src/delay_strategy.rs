/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Retry Delay Strategy
//!
//! Defines the delay calculation methods used in the retry mechanism.
//!
//! # Author
//!
//! Haixing Hu

use rand::Rng;
use std::time::Duration;

/// Retry delay strategy enum
///
/// This enum is used to explicitly specify the delay calculation method used in the retry mechanism, avoiding ambiguity and redundancy in parameter configuration.
/// Each strategy corresponds to different delay calculation logic and parameter requirements.
///
/// # Author
///
/// Haixing Hu
#[derive(Debug, PartialEq)]
pub enum RetryDelayStrategy {
    /// No delay strategy
    ///
    /// Retry immediately without waiting. This is the most aggressive retry strategy, suitable for:
    /// - Very low-latency operation requirements
    /// - Fast retries for atomic operations like CAS (Compare-And-Swap)
    /// - Resolving data races in memory
    /// - Scenarios requiring maximum retry frequency
    ///
    /// **Precautions:**
    /// - May cause CPU resource waste, use with caution
    /// - Not suitable for retrying network or IO operations
    /// - Recommended to use with a lower maximum retry count
    None,

    /// Fixed delay strategy
    ///
    /// Wait the same amount of time between each retry. This is the simplest delay strategy, suitable for:
    /// - Scenarios where error recovery time is relatively fixed
    /// - Low system load situations that don't require yielding
    /// - Need for simple and predictable retry behavior
    Fixed {
        /// Fixed delay time
        delay: Duration,
    },

    /// Random delay strategy
    ///
    /// Wait a random amount of time within the specified range between each retry. Suitable for:
    /// - High-concurrency scenarios requiring avoidance of "thundering herd effect"
    /// - Distributed systems with multiple clients retrying simultaneously
    /// - Scenarios with uncertain error recovery time
    Random {
        /// Minimum value for random delay
        min_delay: Duration,
        /// Maximum value for random delay
        max_delay: Duration,
    },

    /// Exponential backoff strategy
    ///
    /// The delay time for each retry increases exponentially until reaching a maximum value. This is the best strategy for handling system load and
    /// transient failures, suitable for:
    /// - High system load requiring gradual reduction in retry frequency
    /// - Transient failure scenarios for networks or services
    /// - Situations requiring more time for system recovery
    /// - Most distributed systems and microservice architectures
    ///
    /// **Calculation Formula:**
    /// ```text
    /// Next delay = min(current delay × multiplier, max_delay)
    /// ```
    ExponentialBackoff {
        /// Initial delay for exponential backoff
        initial_delay: Duration,
        /// Maximum delay ceiling for exponential backoff
        max_delay: Duration,
        /// Delay multiplier for exponential backoff
        multiplier: f64,
    },
}

impl RetryDelayStrategy {
    /// Create no delay strategy
    ///
    /// Create a no-delay retry strategy that retries immediately without waiting.
    ///
    /// # Returns
    ///
    /// Returns a `RetryDelayStrategy::None` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryDelayStrategy;
    ///
    /// let strategy = RetryDelayStrategy::none();
    /// let delay = strategy.calculate_delay(1, 0.0);
    /// assert_eq!(delay.as_nanos(), 0);
    /// ```
    pub fn none() -> Self {
        RetryDelayStrategy::None
    }

    /// Create fixed delay strategy
    ///
    /// Create a fixed delay retry strategy that waits the same amount of time between each retry.
    ///
    /// # Parameters
    ///
    /// * `delay` - Fixed delay time
    ///
    /// # Returns
    ///
    /// Returns a `RetryDelayStrategy::Fixed` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryDelayStrategy;
    /// use std::time::Duration;
    ///
    /// let strategy = RetryDelayStrategy::fixed(Duration::from_secs(1));
    /// let delay = strategy.calculate_delay(1, 0.0);
    /// assert_eq!(delay, Duration::from_secs(1));
    /// ```
    pub fn fixed(delay: Duration) -> Self {
        RetryDelayStrategy::Fixed { delay }
    }

    /// Create random delay strategy
    ///
    /// Create a random delay retry strategy that waits a random amount of time within the specified range between each retry.
    ///
    /// # Parameters
    ///
    /// * `min_delay` - Minimum value for random delay
    /// * `max_delay` - Maximum value for random delay
    ///
    /// # Returns
    ///
    /// Returns a `RetryDelayStrategy::Random` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryDelayStrategy;
    /// use std::time::Duration;
    ///
    /// let strategy = RetryDelayStrategy::random(
    ///     Duration::from_millis(100),
    ///     Duration::from_millis(1000)
    /// );
    /// let delay = strategy.calculate_delay(1, 0.0);
    /// assert!(delay >= Duration::from_millis(100));
    /// assert!(delay <= Duration::from_millis(1000));
    /// ```
    pub fn random(min_delay: Duration, max_delay: Duration) -> Self {
        RetryDelayStrategy::Random {
            min_delay,
            max_delay,
        }
    }

    /// Create exponential backoff strategy
    ///
    /// Create an exponential backoff retry strategy where the delay time increases exponentially for each retry until reaching a maximum value.
    ///
    /// # Parameters
    ///
    /// * `initial_delay` - Initial delay for exponential backoff
    /// * `max_delay` - Maximum delay ceiling for exponential backoff
    /// * `multiplier` - Delay multiplier for exponential backoff (must be greater than 1.0)
    ///
    /// # Returns
    ///
    /// Returns a `RetryDelayStrategy::ExponentialBackoff` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryDelayStrategy;
    /// use std::time::Duration;
    ///
    /// let strategy = RetryDelayStrategy::exponential_backoff(
    ///     Duration::from_millis(100),
    ///     Duration::from_secs(10),
    ///     2.0
    /// );
    /// let delay1 = strategy.calculate_delay(1, 0.0);
    /// let delay2 = strategy.calculate_delay(2, 0.0);
    /// assert_eq!(delay1, Duration::from_millis(100));
    /// assert_eq!(delay2, Duration::from_millis(200));
    /// ```
    pub fn exponential_backoff(
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
    ) -> Self {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay,
            max_delay,
            multiplier,
        }
    }

    /// Calculate delay time for a specified attempt number
    ///
    /// # Parameters
    ///
    /// * `attempt` - Current attempt number (starting from 1)
    /// * `jitter_factor` - Jitter factor (0.0-1.0) for adding randomness
    ///
    /// # Returns
    ///
    /// Returns the calculated delay time
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryDelayStrategy;
    /// use std::time::Duration;
    ///
    /// let strategy = RetryDelayStrategy::fixed(Duration::from_secs(1));
    /// let delay = strategy.calculate_delay(1, 0.1);
    /// assert!(delay >= Duration::from_millis(900));
    /// assert!(delay <= Duration::from_millis(1100));
    /// ```
    pub fn calculate_delay(&self, attempt: u32, jitter_factor: f64) -> Duration {
        let base_delay = match self {
            RetryDelayStrategy::None => Duration::ZERO,
            RetryDelayStrategy::Fixed { delay } => *delay,
            RetryDelayStrategy::Random {
                min_delay,
                max_delay,
            } => {
                let mut rng = rand::rng();
                let min_nanos = min_delay.as_nanos() as u64;
                let max_nanos = max_delay.as_nanos() as u64;
                let random_nanos = rng.random_range(min_nanos..=max_nanos);
                Duration::from_nanos(random_nanos)
            }
            RetryDelayStrategy::ExponentialBackoff {
                initial_delay,
                max_delay,
                multiplier,
            } => {
                let delay_nanos = initial_delay.as_nanos() as f64;
                let calculated_nanos = delay_nanos * multiplier.powi((attempt - 1) as i32);
                let max_nanos = max_delay.as_nanos() as f64;
                let final_nanos = calculated_nanos.min(max_nanos);
                Duration::from_nanos(final_nanos as u64)
            }
        };

        // Apply jitter
        if jitter_factor > 0.0 && base_delay > Duration::ZERO {
            let mut rng = rand::rng();
            let jitter_range = base_delay.as_nanos() as f64 * jitter_factor;
            let jitter_nanos = rng.random_range(0.0..=jitter_range);
            let total_nanos = base_delay.as_nanos() as f64 + jitter_nanos;
            Duration::from_nanos(total_nanos as u64)
        } else {
            base_delay
        }
    }

    /// Validate the validity of strategy parameters
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if parameters are valid, otherwise returns error message
    pub fn validate(&self) -> Result<(), String> {
        match self {
            RetryDelayStrategy::None => Ok(()),
            RetryDelayStrategy::Fixed { delay } => {
                if delay.is_zero() {
                    Err("Fixed delay cannot be zero".to_string())
                } else {
                    Ok(())
                }
            }
            RetryDelayStrategy::Random {
                min_delay,
                max_delay,
            } => {
                if min_delay.is_zero() {
                    Err("Random delay minimum cannot be zero".to_string())
                } else if *min_delay >= *max_delay {
                    Err("Random delay minimum must be less than maximum".to_string())
                } else {
                    Ok(())
                }
            }
            RetryDelayStrategy::ExponentialBackoff {
                initial_delay,
                max_delay,
                multiplier,
            } => {
                if initial_delay.is_zero() {
                    Err("Exponential backoff initial delay cannot be zero".to_string())
                } else if *initial_delay >= *max_delay {
                    Err(
                        "Exponential backoff initial delay must be less than maximum delay"
                            .to_string(),
                    )
                } else if *multiplier <= 1.0 {
                    Err("Exponential backoff multiplier must be greater than 1.0".to_string())
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl Clone for RetryDelayStrategy {
    /// Clone retry delay strategy
    ///
    /// Create a new instance of the retry delay strategy with the same configuration parameters as the original instance.
    /// Since strategy configuration typically doesn't contain mutable state, cloning is a lightweight operation.
    ///
    /// # Returns
    ///
    /// Returns a cloned instance of the strategy
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryDelayStrategy;
    /// use std::time::Duration;
    ///
    /// let original = RetryDelayStrategy::fixed(Duration::from_secs(1));
    /// let cloned = original.clone();
    /// assert_eq!(original, cloned);
    /// ```
    fn clone(&self) -> Self {
        match self {
            RetryDelayStrategy::None => RetryDelayStrategy::None,
            RetryDelayStrategy::Fixed { delay } => RetryDelayStrategy::Fixed { delay: *delay },
            RetryDelayStrategy::Random {
                min_delay,
                max_delay,
            } => RetryDelayStrategy::Random {
                min_delay: *min_delay,
                max_delay: *max_delay,
            },
            RetryDelayStrategy::ExponentialBackoff {
                initial_delay,
                max_delay,
                multiplier,
            } => RetryDelayStrategy::ExponentialBackoff {
                initial_delay: *initial_delay,
                max_delay: *max_delay,
                multiplier: *multiplier,
            },
        }
    }
}

impl Default for RetryDelayStrategy {
    /// Create default retry delay strategy
    ///
    /// Returns an exponential backoff strategy as the default strategy with the following parameters:
    /// - Initial delay: 1000 milliseconds
    /// - Maximum delay: 60 seconds
    /// - Multiplier: 2.0
    ///
    /// This default configuration is suitable for most distributed systems and microservice architecture scenarios.
    ///
    /// # Returns
    ///
    /// Returns a default exponential backoff strategy instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::RetryDelayStrategy;
    /// use std::time::Duration;
    ///
    /// let strategy = RetryDelayStrategy::default();
    /// let delay = strategy.calculate_delay(1, 0.0);
    /// assert_eq!(delay, Duration::from_millis(1000));
    /// ```
    fn default() -> Self {
        RetryDelayStrategy::ExponentialBackoff {
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
        }
    }
}
