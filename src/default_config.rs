/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Default Retry Configuration Implementation
//!
//! Provides a default retry configuration implementation based on `Config`.
//!
//! # Author
//!
//! Haixing Hu

use super::config::RetryConfig;
use super::delay_strategy::RetryDelayStrategy;
use qubit_config::{Config, Configurable};
use std::time::Duration;

/// Default retry configuration implementation
///
/// Uses `Config` to store all configuration values, implementing both the `RetryConfig` trait and the `Configurable` trait.
///
/// # Author
///
/// Haixing Hu
#[derive(Debug, Clone)]
pub struct DefaultRetryConfig {
    config: Config,
}

impl DefaultRetryConfig {
    /// Create a new `DefaultRetryConfig` instance with default configuration
    pub fn new() -> Self {
        Self {
            config: Config::new(),
        }
    }

    /// Create a new `DefaultRetryConfig` instance with the specified configuration
    pub fn with_config(config: Config) -> Self {
        Self { config }
    }

    /// Get delay strategy from configuration
    fn get_delay_strategy_from_config(&self) -> RetryDelayStrategy {
        let strategy_name = self
            .config
            .get_string_or(Self::KEY_DELAY_STRATEGY, "EXPONENTIAL_BACKOFF");

        match strategy_name.as_str() {
            "NONE" => RetryDelayStrategy::none(),
            "FIXED" => {
                let delay_millis = self
                    .config
                    .get_or(Self::KEY_FIXED_DELAY, Self::DEFAULT_FIXED_DELAY_MILLIS);
                RetryDelayStrategy::fixed(Duration::from_millis(delay_millis))
            }
            "RANDOM" => {
                let min_delay_millis = self.config.get_or(
                    Self::KEY_RANDOM_MIN_DELAY,
                    Self::DEFAULT_RANDOM_MIN_DELAY_MILLIS,
                );
                let max_delay_millis = self.config.get_or(
                    Self::KEY_RANDOM_MAX_DELAY,
                    Self::DEFAULT_RANDOM_MAX_DELAY_MILLIS,
                );
                RetryDelayStrategy::random(
                    Duration::from_millis(min_delay_millis),
                    Duration::from_millis(max_delay_millis),
                )
            }
            "EXPONENTIAL_BACKOFF" => {
                let initial_delay_millis = self.config.get_or(
                    Self::KEY_BACKOFF_INITIAL_DELAY,
                    Self::DEFAULT_BACKOFF_INITIAL_DELAY_MILLIS,
                );
                let max_delay_millis = self.config.get_or(
                    Self::KEY_BACKOFF_MAX_DELAY,
                    Self::DEFAULT_BACKOFF_MAX_DELAY_MILLIS,
                );
                let multiplier = self.config.get_or(
                    Self::KEY_BACKOFF_MULTIPLIER,
                    Self::DEFAULT_BACKOFF_MULTIPLIER,
                );
                RetryDelayStrategy::exponential_backoff(
                    Duration::from_millis(initial_delay_millis),
                    Duration::from_millis(max_delay_millis),
                    multiplier,
                )
            }
            _ => Self::DEFAULT_DELAY_STRATEGY,
        }
    }

    /// Save delay strategy to configuration
    fn set_delay_strategy_to_config(&mut self, strategy: &RetryDelayStrategy) {
        match strategy {
            RetryDelayStrategy::None => {
                self.config.set(Self::KEY_DELAY_STRATEGY, "NONE").unwrap();
            }
            RetryDelayStrategy::Fixed { delay } => {
                self.config.set(Self::KEY_DELAY_STRATEGY, "FIXED").unwrap();
                self.config
                    .set(Self::KEY_FIXED_DELAY, delay.as_millis() as u64)
                    .unwrap();
            }
            RetryDelayStrategy::Random {
                min_delay,
                max_delay,
            } => {
                self.config.set(Self::KEY_DELAY_STRATEGY, "RANDOM").unwrap();
                self.config
                    .set(Self::KEY_RANDOM_MIN_DELAY, min_delay.as_millis() as u64)
                    .unwrap();
                self.config
                    .set(Self::KEY_RANDOM_MAX_DELAY, max_delay.as_millis() as u64)
                    .unwrap();
            }
            RetryDelayStrategy::ExponentialBackoff {
                initial_delay,
                max_delay,
                multiplier,
            } => {
                self.config
                    .set(Self::KEY_DELAY_STRATEGY, "EXPONENTIAL_BACKOFF")
                    .unwrap();
                self.config
                    .set(
                        Self::KEY_BACKOFF_INITIAL_DELAY,
                        initial_delay.as_millis() as u64,
                    )
                    .unwrap();
                self.config
                    .set(Self::KEY_BACKOFF_MAX_DELAY, max_delay.as_millis() as u64)
                    .unwrap();
                self.config
                    .set(Self::KEY_BACKOFF_MULTIPLIER, *multiplier)
                    .unwrap();
            }
        }
    }
}

impl Default for DefaultRetryConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl Configurable for DefaultRetryConfig {
    fn config(&self) -> &Config {
        &self.config
    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    fn set_config(&mut self, config: Config) {
        self.config = config;
        self.on_config_changed();
    }

    fn on_config_changed(&mut self) {
        // Default implementation is empty, subclasses can override this method
    }
}

impl RetryConfig for DefaultRetryConfig {
    fn max_attempts(&self) -> u32 {
        self.config
            .get_or(Self::KEY_MAX_ATTEMPTS, Self::DEFAULT_MAX_ATTEMPTS)
    }

    fn set_max_attempts(&mut self, max_attempts: u32) -> &mut Self {
        self.config
            .set(Self::KEY_MAX_ATTEMPTS, max_attempts)
            .unwrap();
        self
    }

    fn max_duration(&self) -> Option<Duration> {
        let millis = self
            .config
            .get_or(Self::KEY_MAX_DURATION, Self::DEFAULT_MAX_DURATION_MILLIS);
        if millis == 0 {
            None
        } else {
            Some(Duration::from_millis(millis))
        }
    }

    fn set_max_duration(&mut self, max_duration: Option<Duration>) -> &mut Self {
        match max_duration {
            None => self.config.set(Self::KEY_MAX_DURATION, 0u64).unwrap(),
            Some(duration) => self
                .config
                .set(Self::KEY_MAX_DURATION, duration.as_millis() as u64)
                .unwrap(),
        }
        self
    }

    fn operation_timeout(&self) -> Option<Duration> {
        let millis = self.config.get_or(
            Self::KEY_OPERATION_TIMEOUT,
            Self::DEFAULT_OPERATION_TIMEOUT_MILLIS,
        );
        if millis == 0 {
            None
        } else {
            Some(Duration::from_millis(millis))
        }
    }

    fn set_operation_timeout(&mut self, timeout: Option<Duration>) -> &mut Self {
        match timeout {
            None => self.config.set(Self::KEY_OPERATION_TIMEOUT, 0u64).unwrap(),
            Some(duration) => self
                .config
                .set(Self::KEY_OPERATION_TIMEOUT, duration.as_millis() as u64)
                .unwrap(),
        }
        self
    }

    fn delay_strategy(&self) -> RetryDelayStrategy {
        self.get_delay_strategy_from_config()
    }

    fn set_delay_strategy(&mut self, delay_strategy: RetryDelayStrategy) -> &mut Self {
        self.set_delay_strategy_to_config(&delay_strategy);
        self
    }

    fn jitter_factor(&self) -> f64 {
        self.config
            .get_or(Self::KEY_JITTER_FACTOR, Self::DEFAULT_JITTER_FACTOR)
    }

    fn set_jitter_factor(&mut self, jitter_factor: f64) -> &mut Self {
        self.config
            .set(Self::KEY_JITTER_FACTOR, jitter_factor)
            .unwrap();
        self
    }
}
