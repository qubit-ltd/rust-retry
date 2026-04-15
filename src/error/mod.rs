/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Error types used by retry executors.

mod retry_attempt_failure;
mod retry_config_error;
mod retry_decider;
mod retry_error;
mod retry_failure_action;

pub use retry_attempt_failure::RetryAttemptFailure;
pub use retry_config_error::RetryConfigError;
pub use retry_decider::RetryDecider;
pub use retry_error::RetryError;
pub(super) use retry_failure_action::RetryFailureAction;
