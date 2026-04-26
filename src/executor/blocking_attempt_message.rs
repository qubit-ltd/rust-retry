/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Message sent from one blocking attempt worker to the retry executor.

use crate::error::AttemptPanic;

/// Message sent from one blocking attempt worker to the retry executor.
pub(in crate::executor) enum BlockingAttemptMessage<T, E> {
    /// Operation returned normally.
    Result(Result<T, E>),
    /// Operation panicked before timeout.
    Panic(AttemptPanic),
}
