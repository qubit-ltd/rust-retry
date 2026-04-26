/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Boxed future used by a value-erased async attempt.

#[cfg(feature = "tokio")]
use std::future::Future;
#[cfg(feature = "tokio")]
use std::pin::Pin;

use crate::error::AttemptFailure;

/// Boxed future returned by a value-erased async attempt.
#[cfg(feature = "tokio")]
pub(in crate::executor) type AsyncAttemptFuture<'a, E> =
    Pin<Box<dyn Future<Output = Result<(), AttemptFailure<E>>> + 'a>>;
