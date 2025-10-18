/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Retry Event Module
//!
//! Defines various event types and related structures that can occur
//! during the retry process.
//!
//! # Author
//!
//! Haixing Hu

pub mod abort_event;
pub mod abort_reason;
pub mod failure_event;
pub mod retry_decision;
pub mod retry_event;
pub mod retry_reason;
pub mod success_event;

// Re-export main types for convenience
pub use abort_event::{AbortEvent, AbortEventBuilder, AbortEventListener};
pub use abort_reason::AbortReason;
pub use failure_event::{FailureEvent, FailureEventBuilder, FailureEventListener};
pub use retry_decision::RetryDecision;
pub use retry_event::{RetryEvent, RetryEventBuilder, RetryEventListener};
pub use retry_reason::RetryReason;
pub use success_event::{SuccessEvent, SuccessEventBuilder, SuccessEventListener};
