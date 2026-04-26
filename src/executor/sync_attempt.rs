/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Type-erased synchronous attempt used by the retry loop.

use crate::error::AttemptFailure;

/// Type-erased synchronous attempt used by the retry loop.
pub(in crate::executor) trait SyncAttempt<E> {
    /// Calls the wrapped operation once.
    ///
    /// # Returns
    /// `Ok(())` when the operation succeeded, or an attempt failure otherwise.
    fn call(&mut self) -> Result<(), AttemptFailure<E>>;
}
