/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Retry listener type aliases.
//!
//! Listener callbacks are shared through `rs-function` functors so cloned
//! executors invoke the same callback set and can compose listeners with
//! function combinators.

use qubit_function::{ArcBiConsumer, ArcConsumer};

use crate::AttemptFailure;

use super::{AbortContext, FailureContext, RetryContext, SuccessEvent};

/// Listener invoked before sleeping for a retry.
///
/// The callback receives retry metadata and the triggering failure separately.
pub type RetryListener<E> = ArcBiConsumer<RetryContext, AttemptFailure<E>>;

/// Listener invoked when the operation eventually succeeds.
///
/// The callback receives a borrowed [`SuccessEvent`] and is invoked exactly
/// once for a successful executor execution.
pub type SuccessListener = ArcConsumer<SuccessEvent>;

/// Listener invoked when retry limits are exhausted.
///
/// The callback receives failure metadata plus an optional final failure
/// payload (`None` means stopped before the first attempt).
pub type FailureListener<E> = ArcBiConsumer<FailureContext, Option<AttemptFailure<E>>>;

/// Listener invoked when the classifier aborts retrying.
///
/// The callback receives abort metadata and the triggering failure separately.
pub type AbortListener<E> = ArcBiConsumer<AbortContext, AttemptFailure<E>>;

#[derive(Clone)]
pub(crate) struct RetryListeners<E> {
    /// Optional callback invoked before sleeping for a retry.
    pub(crate) retry: Option<RetryListener<E>>,
    /// Optional callback invoked when the operation eventually succeeds.
    pub(crate) success: Option<SuccessListener>,
    /// Optional callback invoked when retry limits are exhausted.
    pub(crate) failure: Option<FailureListener<E>>,
    /// Optional callback invoked when the classifier aborts retrying.
    pub(crate) abort: Option<AbortListener<E>>,
}

impl<E> Default for RetryListeners<E> {
    /// Creates an empty listener set.
    ///
    /// # Parameters
    /// This function has no parameters.
    ///
    /// # Returns
    /// A [`RetryListeners`] value with every callback unset.
    ///
    /// # Errors
    /// This function does not return errors.
    #[inline]
    fn default() -> Self {
        Self {
            retry: None,
            success: None,
            failure: None,
            abort: None,
        }
    }
}
