/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Retry jitter applied on top of a base [`crate::RetryDelay`].
//!
//! After [`crate::RetryDelay`] yields a base sleep duration for the next attempt,
//! [`RetryJitter`] optionally perturbs it so concurrent retries do not align on the
//! same schedule.
//!
//! # Text interchange
//!
//! [`std::fmt::Display`] and [`std::str::FromStr`] use the same grammar:
//!
//! - `none` in any ASCII letter case (leading/trailing ASCII whitespace trimmed).
//! - `factor:` followed by a floating-point literal in **`[0.0, 1.0]`**; optional
//!   ASCII whitespace is allowed after the colon.
//!
//! The `factor:` prefix itself is **case-sensitive**. See
//! [`crate::constants::DEFAULT_RETRY_JITTER`] for the library default string.
//!
//! Author: Haixing Hu

use std::fmt;
use std::num::ParseFloatError;
use std::str::FromStr;
use std::time::Duration;

use rand::RngExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::constants::DEFAULT_RETRY_JITTER;
use crate::RetryDelay;

/// Jitter strategy applied after a base [`crate::RetryDelay`] has been calculated.
///
/// Supports [`RetryJitter::None`] and symmetric [`RetryJitter::Factor`] jitter.
/// After randomization, delays are clamped to **non-negative** values.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RetryJitter {
    /// No jitter: [`RetryJitter::apply`] returns the base delay unchanged.
    None,

    /// Symmetric relative jitter around the base delay.
    ///
    /// The inner `f64` is the relative half-span: jitter is drawn uniformly from
    /// `[-base * factor, base * factor]` nanoseconds (see [`RetryJitter::apply`]).
    /// It must be finite and lie in **`[0.0, 1.0]`** for validated configurations.
    Factor(f64),
}

/// Failure to parse a [`RetryJitter`] from its text form ([`std::str::FromStr`]).
#[derive(Debug, Error)]
pub enum ParseRetryJitterError {
    /// After [`str::trim`], the input is neither `none` in any ASCII letter case nor
    /// a string beginning with the literal ASCII prefix `factor:`.
    #[error("invalid retry jitter format, expected `none` or `factor:<number>`")]
    InvalidFormat,

    /// The substring after `factor:` is not a valid `f64` (after trimming).
    #[error("invalid retry jitter factor")]
    InvalidNumber(#[from] ParseFloatError),

    /// Parsed factor is not in the inclusive interval **`[0.0, 1.0]`** (includes
    /// non-finite values such as NaN and infinity, which fail the range check).
    #[error("retry jitter factor must be in range [0.0, 1.0], got {value}")]
    OutOfRange { value: f64 },
}

impl RetryJitter {
    /// Creates a no-jitter strategy.
    ///
    /// # Parameters
    /// This function has no parameters.
    ///
    /// # Returns
    /// A [`RetryJitter::None`] strategy.
    ///
    /// # Errors
    /// This function does not return errors.
    #[inline]
    pub fn none() -> Self {
        Self::None
    }

    /// Creates a symmetric relative jitter strategy.
    ///
    /// Validation requires `factor` to be finite and within `[0.0, 1.0]`.
    ///
    /// # Parameters
    /// - `factor`: Relative jitter range. For example, `0.2` samples from
    ///   `base +/- 20%`.
    ///
    /// # Returns
    /// A [`RetryJitter::Factor`] strategy.
    ///
    /// # Errors
    /// This constructor does not validate `factor`; use [`RetryJitter::validate`]
    /// before applying values that come from configuration or user input.
    #[inline]
    pub fn factor(factor: f64) -> Self {
        Self::Factor(factor)
    }

    /// Applies jitter to a base delay.
    ///
    /// For [`RetryJitter::None`], returns `base` unchanged.
    ///
    /// For [`RetryJitter::Factor`], if `factor <= 0.0` or `base` is zero, returns
    /// `base` unchanged. Otherwise draws a uniform sample from the inclusive range
    /// `[-base * factor, base * factor]` in nanosecond space, adds it to the base,
    /// then clamps the result to **at least zero** (truncating the sum to `u64`
    /// nanoseconds).
    ///
    /// # Parameters
    /// - `base`: Base delay calculated by [`crate::RetryDelay`].
    ///
    /// # Returns
    /// The jittered delay, never below zero.
    ///
    /// # Errors
    /// This function does not return errors.
    ///
    /// # Panics
    /// May panic if a [`RetryJitter::Factor`] value has not been validated and the
    /// factor is non-finite, because the random range cannot be sampled.
    pub fn apply(&self, base: Duration) -> Duration {
        match self {
            Self::None => base,
            Self::Factor(factor) if *factor <= 0.0 || base.is_zero() => base,
            Self::Factor(factor) => {
                let base_nanos = base.as_nanos() as f64;
                let span = base_nanos * factor;
                let mut rng = rand::rng();
                let jitter = rng.random_range(-span..=span);
                Duration::from_nanos((base_nanos + jitter).max(0.0) as u64)
            }
        }
    }

    /// Calculates and jitters the delay for one retry attempt.
    ///
    /// This method combines base-delay strategy selection and jitter application
    /// into one step.
    ///
    /// # Parameters
    /// - `delay_strategy`: Base delay strategy used to calculate the attempt
    ///   delay.
    /// - `attempt`: Failed-attempt index passed to
    ///   [`RetryDelay::base_delay`].
    ///
    /// # Returns
    /// The delay for the attempt after jitter is applied.
    ///
    /// # Errors
    /// This function does not return errors.
    ///
    /// # Panics
    /// May panic if a [`RetryJitter::Factor`] value has not been validated and
    /// its factor is non-finite.
    pub fn delay_for_attempt(&self, delay_strategy: &RetryDelay, attempt: u32) -> Duration {
        let base_delay = delay_strategy.base_delay(attempt);
        self.apply(base_delay)
    }

    /// Validates jitter parameters for use with executors and options.
    ///
    /// [`RetryJitter::None`] is always valid. For [`RetryJitter::Factor`], the factor
    /// must be finite and satisfy **`0.0 <= factor <= 1.0`** (endpoints included).
    ///
    /// # Returns
    /// `Ok(())` when the jitter configuration is usable.
    ///
    /// # Parameters
    /// This method has no parameters.
    ///
    /// # Errors
    /// Returns an error when the factor is negative, greater than `1.0`, NaN,
    /// or infinite.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::None => Ok(()),
            Self::Factor(factor) => {
                if !factor.is_finite() || *factor < 0.0 || *factor > 1.0 {
                    Err("jitter factor must be finite and in range [0.0, 1.0]".to_string())
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl Default for RetryJitter {
    /// Creates the default jitter strategy.
    ///
    /// # Returns
    /// The value obtained by parsing [`crate::constants::DEFAULT_RETRY_JITTER`]
    /// using [`RetryJitter::from_str`].
    ///
    /// # Parameters
    /// This function has no parameters.
    ///
    /// # Errors
    /// This function does not return errors.
    ///
    /// # Panics
    /// Panics if [`crate::constants::DEFAULT_RETRY_JITTER`] is not a valid
    /// [`RetryJitter`] string. That indicates a crate bug, not a caller mistake.
    #[inline]
    fn default() -> Self {
        Self::from_str(DEFAULT_RETRY_JITTER)
            .expect("DEFAULT_RETRY_JITTER must be a valid RetryJitter string")
    }
}

impl fmt::Display for RetryJitter {
    /// Writes the canonical text form: `none`, or `factor:` followed by the factor
    /// using the default `f64` formatter.
    ///
    /// # Parameters
    /// - `f`: Output formatter.
    ///
    /// # Returns
    /// `Ok(())` on success, or [`fmt::Error`] if the formatter rejects output.
    ///
    /// # Errors
    /// Returns [`fmt::Error`] only when the underlying [`write!`] fails.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Factor(v) => write!(f, "factor:{v}"),
        }
    }
}

impl FromStr for RetryJitter {
    /// Parse error type for invalid or out-of-range text.
    type Err = ParseRetryJitterError;

    /// Parses a [`RetryJitter`] from a single token string.
    ///
    /// Leading and trailing Unicode whitespace is removed with [`str::trim`]. The
    /// `none` branch matches with [`str::eq_ignore_ascii_case`]. The `factor:` prefix
    /// must match exactly in ASCII case; the number may be surrounded by ASCII
    /// whitespace after the colon.
    ///
    /// # Parameters
    /// - `s`: Input text (for example from configuration or CLI).
    ///
    /// # Returns
    /// `Ok(Self)` on success, or [`ParseRetryJitterError`] describing the failure.
    ///
    /// # Errors
    /// - [`ParseRetryJitterError::InvalidFormat`] when the token is neither `none`
    ///   (ASCII case-insensitive) nor `factor:` followed by a number.
    /// - [`ParseRetryJitterError::InvalidNumber`] when the factor is not a valid
    ///   `f64`.
    /// - [`ParseRetryJitterError::OutOfRange`] when the parsed factor is outside
    ///   **`[0.0, 1.0]`** or is non-finite (NaN / infinity).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.eq_ignore_ascii_case("none") {
            return Ok(Self::None);
        }

        let Some(raw) = s.strip_prefix("factor:") else {
            return Err(ParseRetryJitterError::InvalidFormat);
        };

        let value: f64 = raw.trim().parse()?;

        if !(0.0..=1.0).contains(&value) {
            return Err(ParseRetryJitterError::OutOfRange { value });
        }

        Ok(Self::Factor(value))
    }
}
