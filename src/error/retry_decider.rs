/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use crate::event::{RetryAttemptContext, RetryDecision};
use qubit_function::ArcBiFunction;

/// Decides whether to perform another retry after a failed attempt.
///
/// The executor calls this with the application **error** `E` and a
/// [`RetryAttemptContext`] snapshot; implementors inspect that error (and
/// context) and return [`RetryDecision::Retry`] to try again or
/// [`RetryDecision::Abort`] to stop immediately with [`crate::RetryError::Aborted`],
/// subject to attempt and elapsed-time limits.
///
/// Stored as an [`ArcBiFunction`] so cloned [`crate::RetryExecutor`] instances can
/// share the same logic safely.
pub type RetryDecider<E> = ArcBiFunction<E, RetryAttemptContext, RetryDecision>;
