/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Retry Configuration Interface
//!
//! Defines the configuration interface for the retry mechanism.
//!
//! # Author
//!
//! Haixing Hu

use super::delay_strategy::RetryDelayStrategy;
use std::time::Duration;

/// Retry configuration interface
///
/// This interface defines various configuration options for the retry mechanism, supporting flexible retry strategy configuration.
/// The retry mechanism can be controlled through the following dimensions:
///
/// ## Basic Retry Configuration
/// - **Max Attempts (max_attempts)**: Controls the total number of attempts. Note that the attempt count includes the initial execution,
///   so retry count = attempt count - 1. For example, setting it to 3 means at most 3 attempts, i.e., at most 2 retries.
/// - **Max Duration (max_duration)**: Controls the maximum total time for the entire retry process.
///   Retries will stop after this time, even if the maximum attempt count has not been reached.
///
/// ## Delay Strategy Configuration
/// Explicitly specify the delay strategy to use via the `RetryDelayStrategy` enum, avoiding ambiguity in parameter configuration:
/// - **Fixed Delay (FIXED)**: Use the same delay time for each retry
/// - **Random Delay (RANDOM)**: Use a random delay time within the specified range for each retry
/// - **Exponential Backoff (EXPONENTIAL_BACKOFF)**: Delay time grows exponentially
///
/// ## Jitter Configuration
/// - **Jitter Factor (jitter_factor)**: Relative jitter ratio. Based on the calculated delay time,
///   randomly add 0 to (delay time × jitter_factor) additional wait time to avoid "thundering herd effect".
///
/// # Author
///
/// Haixing Hu
pub trait RetryConfig {
    // --- Configuration key constants ---

    /// Configuration key for maximum number of attempts
    const KEY_MAX_ATTEMPTS: &'static str = "retry.max_attempts";
    /// Configuration key for delay strategy
    const KEY_DELAY_STRATEGY: &'static str = "retry.delay_strategy";
    /// Configuration key for fixed delay time
    const KEY_FIXED_DELAY: &'static str = "retry.fixed_delay_millis";
    /// Configuration key for random delay minimum value
    const KEY_RANDOM_MIN_DELAY: &'static str = "retry.random_min_delay_millis";
    /// Configuration key for random delay maximum value
    const KEY_RANDOM_MAX_DELAY: &'static str = "retry.random_max_delay_millis";
    /// Configuration key for exponential backoff initial delay
    const KEY_BACKOFF_INITIAL_DELAY: &'static str = "retry.backoff_initial_delay_millis";
    /// Configuration key for exponential backoff maximum delay
    const KEY_BACKOFF_MAX_DELAY: &'static str = "retry.backoff_max_delay_millis";
    /// Configuration key for exponential backoff multiplier
    const KEY_BACKOFF_MULTIPLIER: &'static str = "retry.backoff_multiplier";
    /// Configuration key for jitter factor
    const KEY_JITTER_FACTOR: &'static str = "retry.jitter_factor";
    /// Configuration key for maximum duration of retry execution
    const KEY_MAX_DURATION: &'static str = "retry.max_duration_millis";
    /// Configuration key for single operation timeout
    const KEY_OPERATION_TIMEOUT: &'static str = "retry.operation_timeout_millis";

    // --- Default value constants ---

    /// Default maximum number of attempts for retry mechanism
    const DEFAULT_MAX_ATTEMPTS: u32 = 5;
    /// Default delay strategy
    const DEFAULT_DELAY_STRATEGY: RetryDelayStrategy = RetryDelayStrategy::ExponentialBackoff {
        initial_delay: Duration::from_millis(1000),
        max_delay: Duration::from_secs(60),
        multiplier: 2.0,
    };
    /// Default fixed delay time in milliseconds
    const DEFAULT_FIXED_DELAY_MILLIS: u64 = 1000;
    /// Default random delay minimum value in milliseconds
    const DEFAULT_RANDOM_MIN_DELAY_MILLIS: u64 = 1000;
    /// Default random delay maximum value in milliseconds
    const DEFAULT_RANDOM_MAX_DELAY_MILLIS: u64 = 10000;
    /// Default exponential backoff initial delay in milliseconds
    const DEFAULT_BACKOFF_INITIAL_DELAY_MILLIS: u64 = 1000;
    /// Default exponential backoff maximum delay in milliseconds
    const DEFAULT_BACKOFF_MAX_DELAY_MILLIS: u64 = 60000;
    /// Default exponential backoff multiplier
    const DEFAULT_BACKOFF_MULTIPLIER: f64 = 2.0;
    /// Default value for jitter factor
    const DEFAULT_JITTER_FACTOR: f64 = 0.0;
    /// Default maximum duration for retry execution in milliseconds, 0 means unlimited
    const DEFAULT_MAX_DURATION_MILLIS: u64 = 0;
    /// Default value for single operation timeout in milliseconds, 0 means unlimited
    const DEFAULT_OPERATION_TIMEOUT_MILLIS: u64 = 0;

    // --- Basic retry control methods ---

    /// Get maximum number of attempts
    ///
    /// This property controls the total number of attempts for the operation, including the initial execution and all retries. For example:
    /// - Set to 1: Execute only once, no retries
    /// - Set to 3: Execute at most 3 times (initial + at most 2 retries)
    /// - Set to 5: Execute at most 5 times (initial + at most 4 retries)
    /// - Note: Actual retry count = max attempts - 1
    fn max_attempts(&self) -> u32;

    /// Set maximum number of attempts
    fn set_max_attempts(&mut self, max_attempts: u32) -> &mut Self;

    /// Get maximum duration for retry execution
    ///
    /// The maximum duration controls the total time limit for the entire retry process (from initial execution to the last retry).
    /// When the maximum duration is reached, the retry process will stop regardless of whether the maximum attempt count has been reached.
    fn max_duration(&self) -> Option<Duration>;

    /// Set maximum duration for retry execution
    fn set_max_duration(&mut self, max_duration: Option<Duration>) -> &mut Self;

    /// Get single operation timeout
    ///
    /// Single operation timeout controls the maximum time for each operation execution. This differs from max_duration:
    /// - operation_timeout: Maximum execution time for a single operation
    /// - max_duration: Total time for the entire retry process (including all retries and delays)
    ///
    /// Returns None to indicate no single operation timeout limit.
    fn operation_timeout(&self) -> Option<Duration>;

    /// Set single operation timeout
    ///
    /// # Parameters
    ///
    /// * `timeout` - Timeout duration, None means unlimited
    fn set_operation_timeout(&mut self, timeout: Option<Duration>) -> &mut Self;

    // --- Delay strategy methods ---

    /// Get delay strategy type
    fn delay_strategy(&self) -> RetryDelayStrategy;

    /// Set delay strategy type
    fn set_delay_strategy(&mut self, delay_strategy: RetryDelayStrategy) -> &mut Self;

    /// Get jitter factor
    ///
    /// The jitter factor is a percentage jitter relative to the current delay time. The jitter factor's effective range is [0, delay time × jitterFactor],
    /// meaning actual delay = calculated delay + random(0, delay time × jitter factor).
    fn jitter_factor(&self) -> f64;

    /// Set jitter factor
    fn set_jitter_factor(&mut self, jitter_factor: f64) -> &mut Self;

    // --- Convenience methods ---

    /// Set random delay range
    ///
    /// This is a convenience method that sets both the minimum and maximum values for random delay, and sets the delay strategy to random delay.
    fn set_random_delay_strategy(&mut self, min_delay: Duration, max_delay: Duration) -> &mut Self {
        self.set_delay_strategy(RetryDelayStrategy::random(min_delay, max_delay));
        self
    }

    /// Set fixed delay
    ///
    /// This is a convenience method that sets the fixed delay time and sets the delay strategy to fixed delay.
    fn set_fixed_delay_strategy(&mut self, delay: Duration) -> &mut Self {
        self.set_delay_strategy(RetryDelayStrategy::fixed(delay));
        self
    }

    /// Set exponential backoff strategy parameters
    ///
    /// This is a convenience method that sets the initial delay, maximum delay, and multiplier for exponential backoff,
    /// and sets the delay strategy to exponential backoff.
    fn set_exponential_backoff_strategy(
        &mut self,
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
    ) -> &mut Self {
        self.set_delay_strategy(RetryDelayStrategy::exponential_backoff(
            initial_delay,
            max_delay,
            multiplier,
        ));
        self
    }

    /// Set no delay strategy
    ///
    /// This is a convenience method that sets the delay strategy to no delay.
    fn set_no_delay_strategy(&mut self) -> &mut Self {
        self.set_delay_strategy(RetryDelayStrategy::none());
        self
    }

    /// Set unlimited duration
    ///
    /// This is a convenience method that sets the maximum duration to None, indicating that the retry process has no time limit,
    /// and is only controlled by the maximum attempt count.
    fn set_unlimited_duration(&mut self) -> &mut Self {
        self.set_max_duration(None);
        self
    }

    /// Set unlimited operation timeout
    ///
    /// This is a convenience method that sets the single operation timeout to None, indicating that a single operation has no time limit.
    fn set_unlimited_operation_timeout(&mut self) -> &mut Self {
        self.set_operation_timeout(None);
        self
    }
}
