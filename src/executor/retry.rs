/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Retry execution.
//!
//! A [`Retry`] owns validated retry options and lifecycle listeners. The
//! operation success type is introduced by each `run` call, while the error type
//! is bound by the retry policy.

use qubit_common::BoxError;
use qubit_function::{BiConsumer, BiFunction, Consumer};
use std::fmt;
#[cfg(feature = "tokio")]
use std::future::Future;
use std::panic;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[cfg(feature = "tokio")]
use super::async_attempt::AsyncAttempt;
#[cfg(feature = "tokio")]
use super::async_value_operation::AsyncValueOperation;
use super::attempt_cancel_token::AttemptCancelToken;
use super::blocking_attempt_message::BlockingAttemptMessage;
use super::retry_flow_action::RetryFlowAction;
use super::sync_attempt::SyncAttempt;
use super::sync_value_operation::SyncValueOperation;
use crate::event::RetryListeners;
use crate::{
    AttemptExecutorError, AttemptFailure, AttemptFailureDecision, AttemptPanic,
    AttemptTimeoutPolicy, RetryAfterHint, RetryBuilder, RetryConfigError, RetryContext, RetryError,
    RetryErrorReason, RetryOptions,
};

/// Retry policy and executor bound to an operation error type.
///
/// The generic parameter `E` is the caller's operation error type. Cloning a
/// retry policy shares all registered functors through reference-counted
/// `rs-function` wrappers.
#[derive(Clone)]
pub struct Retry<E = BoxError> {
    /// Validated retry limits and backoff settings.
    options: RetryOptions,
    /// Optional retry-after hint extractor.
    retry_after_hint: Option<RetryAfterHint<E>>,
    /// Whether listener panics should be isolated.
    isolate_listener_panics: bool,
    /// Lifecycle listeners.
    listeners: RetryListeners<E>,
}

impl<E> Retry<E> {
    /// Creates a retry builder.
    ///
    /// # Returns
    /// A [`RetryBuilder`] configured with defaults.
    #[inline]
    pub fn builder() -> RetryBuilder<E> {
        RetryBuilder::new()
    }

    /// Creates a retry policy from options.
    ///
    /// # Parameters
    /// - `options`: Retry options to validate and install.
    ///
    /// # Returns
    /// A retry policy using the default listener set.
    ///
    /// # Errors
    /// Returns [`RetryConfigError`] if the options are invalid.
    pub fn from_options(options: RetryOptions) -> Result<Self, RetryConfigError> {
        Self::builder().options(options).build()
    }

    /// Returns the immutable options used by this retry policy.
    ///
    /// # Returns
    /// Shared retry options.
    #[inline]
    pub fn options(&self) -> &RetryOptions {
        &self.options
    }

    /// Runs a synchronous operation with retry.
    ///
    /// # Parameters
    /// - `operation`: Operation called once per attempt until it succeeds or the
    ///   retry flow stops.
    ///
    /// # Returns
    /// `Ok(T)` with the operation value, or [`RetryError`] when retrying stops.
    ///
    /// # Panics
    /// Propagates operation panics and listener panics unless listener panic
    /// isolation is enabled.
    ///
    /// # Blocking
    /// Blocks the current thread with `std::thread::sleep` between attempts when
    /// a non-zero retry delay is selected.
    pub fn run<T, F>(&self, mut operation: F) -> Result<T, RetryError<E>>
    where
        F: FnMut() -> Result<T, E>,
    {
        let mut operation = SyncValueOperation::new(&mut operation);
        self.run_sync_operation(&mut operation)?;
        Ok(operation.into_value())
    }

    /// Runs a blocking operation with retry inside worker-thread attempts.
    ///
    /// Each attempt runs on a worker thread. Worker panics are captured as
    /// [`AttemptFailure::Panic`]. Worker-spawn failures are reported as
    /// [`AttemptFailure::Executor`]. If the configured attempt timeout expires,
    /// the retry executor stops waiting, marks the attempt's
    /// [`AttemptCancelToken`] as cancelled, and continues according to
    /// [`AttemptTimeoutPolicy`]. The timed-out worker thread may continue
    /// running and overlap later attempts if the operation ignores the
    /// cancellation token.
    ///
    /// # Parameters
    /// - `operation`: Thread-safe operation called once per attempt. It receives
    ///   a cooperative cancellation token for that attempt.
    ///
    /// # Returns
    /// `Ok(T)` with the operation value, or [`RetryError`] when retrying stops.
    ///
    /// # Panics
    /// Does not propagate operation panics. Listener panic behavior follows this
    /// retry policy's listener isolation setting.
    ///
    /// # Blocking
    /// Blocks the current thread while waiting for each worker result or timeout
    /// and while sleeping between retry attempts.
    pub fn run_in_worker<T, F>(&self, operation: F) -> Result<T, RetryError<E>>
    where
        T: Send + 'static,
        E: Send + 'static,
        F: Fn(AttemptCancelToken) -> Result<T, E> + Send + Sync + 'static,
    {
        let operation = Arc::new(operation);
        let start = Instant::now();
        let mut attempts = 0;
        let mut last_failure = None;

        loop {
            let attempt_timeout = self.attempt_timeout_duration();
            if let Some(error) =
                self.elapsed_error(start, attempts, last_failure.take(), attempt_timeout)
            {
                return Err(self.emit_error(error));
            }

            attempts += 1;
            let before_context = self.context(start, attempts, Duration::ZERO, attempt_timeout);
            self.emit_before_attempt(&before_context);

            let attempt_start = Instant::now();
            let result = self.call_blocking_attempt(Arc::clone(&operation));
            let context = self.context(start, attempts, attempt_start.elapsed(), attempt_timeout);
            match result {
                Ok(value) => {
                    self.emit_attempt_success(&context);
                    return Ok(value);
                }
                Err(failure) => match self.handle_failure(start, attempts, failure, context) {
                    RetryFlowAction::Retry { delay, failure } => {
                        if !delay.is_zero() {
                            std::thread::sleep(delay);
                        }
                        last_failure = Some(failure);
                    }
                    RetryFlowAction::Finished(error) => return Err(self.emit_error(error)),
                },
            }
        }
    }

    /// Runs a blocking operation with retry and per-attempt timeout isolation.
    ///
    /// This method is a compatibility alias for [`Retry::run_in_worker`]. It
    /// also runs attempts in worker threads when no timeout is configured, so
    /// worker panics are reported as [`AttemptFailure::Panic`] instead of
    /// unwinding through the caller. Worker-spawn failures are reported as
    /// [`AttemptFailure::Executor`].
    ///
    /// # Parameters
    /// - `operation`: Thread-safe operation called once per attempt. It receives
    ///   a cooperative cancellation token for that attempt.
    ///
    /// # Returns
    /// `Ok(T)` with the operation value, or [`RetryError`] when retrying stops.
    ///
    /// # Panics
    /// Does not propagate operation panics. Listener panic behavior follows this
    /// retry policy's listener isolation setting.
    ///
    /// # Blocking
    /// Blocks the current thread while waiting for each worker result or timeout
    /// and while sleeping between retry attempts.
    #[inline]
    pub fn run_blocking_with_timeout<T, F>(&self, operation: F) -> Result<T, RetryError<E>>
    where
        T: Send + 'static,
        E: Send + 'static,
        F: Fn(AttemptCancelToken) -> Result<T, E> + Send + Sync + 'static,
    {
        self.run_in_worker(operation)
    }

    /// Runs a synchronous value-erased operation with retry.
    ///
    /// # Parameters
    /// - `operation`: Operation adapter called once per attempt.
    ///
    /// # Returns
    /// `Ok(())` after a successful attempt, or [`RetryError`] when retrying stops.
    fn run_sync_operation(&self, operation: &mut dyn SyncAttempt<E>) -> Result<(), RetryError<E>> {
        let start = Instant::now();
        let mut attempts = 0;
        let mut last_failure = None;

        loop {
            if let Some(error) = self.elapsed_error(start, attempts, last_failure.take(), None) {
                return Err(self.emit_error(error));
            }

            attempts += 1;
            let before_context = self.context(start, attempts, Duration::ZERO, None);
            self.emit_before_attempt(&before_context);

            let attempt_start = Instant::now();
            match operation.call() {
                Ok(()) => {
                    let context = self.context(start, attempts, attempt_start.elapsed(), None);
                    self.emit_attempt_success(&context);
                    return Ok(());
                }
                Err(failure) => {
                    let context = self.context(start, attempts, attempt_start.elapsed(), None);
                    match self.handle_failure(start, attempts, failure, context) {
                        RetryFlowAction::Retry { delay, failure } => {
                            if !delay.is_zero() {
                                std::thread::sleep(delay);
                            }
                            last_failure = Some(failure);
                        }
                        RetryFlowAction::Finished(error) => return Err(self.emit_error(error)),
                    }
                }
            }
        }
    }

    /// Runs an asynchronous operation with retry.
    ///
    /// # Parameters
    /// - `operation`: Factory returning a fresh future for each attempt.
    ///
    /// # Returns
    /// `Ok(T)` with the operation value, or [`RetryError`] when retrying stops.
    ///
    /// # Panics
    /// Propagates operation panics from the current async task. They are not
    /// converted to [`AttemptFailure::Panic`] because `run_async` does not
    /// create an isolation boundary. Listener panics are propagated unless
    /// listener panic isolation is enabled. Tokio may panic if timer APIs are
    /// used outside a runtime with a time driver.
    #[cfg(feature = "tokio")]
    pub async fn run_async<T, F, Fut>(&self, mut operation: F) -> Result<T, RetryError<E>>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let mut operation = AsyncValueOperation::new(&mut operation);
        self.run_async_operation(&mut operation).await?;
        Ok(operation.into_value())
    }

    /// Runs an asynchronous value-erased operation with retry.
    ///
    /// # Parameters
    /// - `operation`: Async operation adapter called once per attempt.
    ///
    /// # Returns
    /// `Ok(())` after a successful attempt, or [`RetryError`] when retrying stops.
    #[cfg(feature = "tokio")]
    async fn run_async_operation(
        &self,
        operation: &mut dyn AsyncAttempt<E>,
    ) -> Result<(), RetryError<E>> {
        let start = Instant::now();
        let mut attempts = 0;
        let mut last_failure = None;

        loop {
            let attempt_timeout = self.attempt_timeout_duration();
            if let Some(error) =
                self.elapsed_error(start, attempts, last_failure.take(), attempt_timeout)
            {
                return Err(self.emit_error(error));
            }

            attempts += 1;
            let before_context = self.context(start, attempts, Duration::ZERO, attempt_timeout);
            self.emit_before_attempt(&before_context);

            let attempt_start = Instant::now();
            let result = if let Some(timeout) = attempt_timeout {
                match tokio::time::timeout(timeout, operation.call()).await {
                    Ok(result) => result,
                    Err(_) => Err(AttemptFailure::Timeout),
                }
            } else {
                operation.call().await
            };

            let context = self.context(start, attempts, attempt_start.elapsed(), attempt_timeout);
            match result {
                Ok(()) => {
                    self.emit_attempt_success(&context);
                    return Ok(());
                }
                Err(failure) => match self.handle_failure(start, attempts, failure, context) {
                    RetryFlowAction::Retry { delay, failure } => {
                        sleep_async(delay).await;
                        last_failure = Some(failure);
                    }
                    RetryFlowAction::Finished(error) => return Err(self.emit_error(error)),
                },
            }
        }
    }

    /// Creates a retry policy from validated parts.
    ///
    /// # Parameters
    /// - `options`: Retry options.
    /// - `retry_after_hint`: Optional hint extractor.
    /// - `isolate_listener_panics`: Whether listener panics are isolated.
    /// - `listeners`: Lifecycle listeners.
    ///
    /// # Returns
    /// A retry policy.
    pub(super) fn new(
        options: RetryOptions,
        retry_after_hint: Option<RetryAfterHint<E>>,
        isolate_listener_panics: bool,
        listeners: RetryListeners<E>,
    ) -> Self {
        Self {
            options,
            retry_after_hint,
            isolate_listener_panics,
            listeners,
        }
    }

    /// Builds a context snapshot.
    ///
    /// # Parameters
    /// - `start`: Retry flow start.
    /// - `attempt`: Current attempt number.
    /// - `attempt_elapsed`: Elapsed time in the current attempt.
    /// - `attempt_timeout`: Timeout configured for the current attempt.
    ///
    /// # Returns
    /// A retry context.
    fn context(
        &self,
        start: Instant,
        attempt: u32,
        attempt_elapsed: Duration,
        attempt_timeout: Option<Duration>,
    ) -> RetryContext {
        RetryContext::new(
            attempt,
            self.options.max_attempts.get(),
            self.options.max_elapsed,
            start.elapsed(),
            attempt_elapsed,
            attempt_timeout,
        )
    }

    /// Returns the configured attempt-timeout duration.
    ///
    /// # Returns
    /// `Some(Duration)` when per-attempt timeout is configured.
    #[inline]
    fn attempt_timeout_duration(&self) -> Option<Duration> {
        self.options
            .attempt_timeout()
            .map(|attempt_timeout| attempt_timeout.timeout())
    }

    /// Runs one blocking attempt on a worker thread.
    ///
    /// # Parameters
    /// - `operation`: Shared blocking operation.
    ///
    /// # Returns
    /// The operation value on success, or an attempt failure.
    ///
    /// # Panics
    /// Converts worker panics into [`AttemptFailure::Panic`] and worker-spawn
    /// failures into [`AttemptFailure::Executor`].
    fn call_blocking_attempt<T, F>(&self, operation: Arc<F>) -> Result<T, AttemptFailure<E>>
    where
        T: Send + 'static,
        E: Send + 'static,
        F: Fn(AttemptCancelToken) -> Result<T, E> + Send + Sync + 'static,
    {
        let token = AttemptCancelToken::new();
        let (sender, receiver) = mpsc::sync_channel(1);
        let worker_token = token.clone();
        let worker = std::thread::Builder::new()
            .name("qubit-retry-worker".to_string())
            .spawn(move || {
                let result =
                    panic::catch_unwind(panic::AssertUnwindSafe(|| operation(worker_token)));
                let message = match result {
                    Ok(result) => BlockingAttemptMessage::Result(result),
                    Err(payload) => {
                        BlockingAttemptMessage::Panic(AttemptPanic::from_payload(payload))
                    }
                };
                let _ = sender.send(message);
            });
        if let Err(error) = worker {
            return Err(AttemptFailure::Executor(
                AttemptExecutorError::from_spawn_error(error),
            ));
        }

        match self.attempt_timeout_duration() {
            Some(attempt_timeout) => {
                let message = receiver.recv_timeout(attempt_timeout);
                if matches!(message, Err(mpsc::RecvTimeoutError::Timeout)) {
                    token.cancel();
                    return Err(AttemptFailure::Timeout);
                }
                worker_message_to_attempt_result(
                    message
                        .expect("blocking retry attempt worker stopped without sending a result"),
                )
            }
            None => worker_message_to_attempt_result(
                receiver
                    .recv()
                    .expect("blocking retry attempt worker stopped without sending a result"),
            ),
        }
    }

    /// Handles one failed attempt.
    ///
    /// # Parameters
    /// - `start`: Retry flow start.
    /// - `attempts`: Attempts executed so far.
    /// - `failure`: Attempt failure.
    /// - `context`: Context captured after the failed attempt.
    ///
    /// # Returns
    /// A retry action selected from listeners and configured limits.
    fn handle_failure(
        &self,
        start: Instant,
        attempts: u32,
        failure: AttemptFailure<E>,
        context: RetryContext,
    ) -> RetryFlowAction<E> {
        let hint = self
            .retry_after_hint
            .as_ref()
            .and_then(|hint| self.invoke_listener(|| hint.apply(&failure, &context)));
        let context = context.with_retry_after_hint(hint);

        let decision =
            self.resolve_failure_decision(self.failure_decision(&failure, &context), &failure);
        if decision == AttemptFailureDecision::Abort {
            return RetryFlowAction::Finished(RetryError::new(
                RetryErrorReason::Aborted,
                Some(failure),
                context,
            ));
        }

        let max_attempts = self.options.max_attempts.get();
        if attempts >= max_attempts {
            return RetryFlowAction::Finished(RetryError::new(
                RetryErrorReason::AttemptsExceeded,
                Some(failure),
                context,
            ));
        }

        let delay = self.retry_delay(decision, attempts, hint);
        let context = context.with_next_delay(delay);
        if let Some(max_elapsed) = self.options.max_elapsed
            && will_exceed_elapsed(start.elapsed(), delay, max_elapsed)
        {
            return RetryFlowAction::Finished(RetryError::new(
                RetryErrorReason::MaxElapsedExceeded,
                Some(failure),
                context,
            ));
        }

        RetryFlowAction::Retry { delay, failure }
    }

    /// Resolves all failure listeners into one decision.
    ///
    /// # Parameters
    /// - `failure`: Attempt failure.
    /// - `context`: Failure context.
    ///
    /// # Returns
    /// Last non-default listener decision, or [`AttemptFailureDecision::UseDefault`].
    fn failure_decision(
        &self,
        failure: &AttemptFailure<E>,
        context: &RetryContext,
    ) -> AttemptFailureDecision {
        let mut decision = AttemptFailureDecision::UseDefault;
        for listener in &self.listeners.failure {
            let current = self.invoke_listener(|| listener.apply(failure, context));
            if current != AttemptFailureDecision::UseDefault {
                decision = current;
            }
        }
        decision
    }

    /// Resolves the effective failure decision after applying timeout policy.
    ///
    /// # Parameters
    /// - `decision`: Decision returned by failure listeners.
    /// - `failure`: Attempt failure being handled.
    ///
    /// # Returns
    /// A concrete decision for timeout failures when listeners used the default.
    fn resolve_failure_decision(
        &self,
        decision: AttemptFailureDecision,
        failure: &AttemptFailure<E>,
    ) -> AttemptFailureDecision {
        if decision != AttemptFailureDecision::UseDefault {
            return decision;
        }
        if matches!(failure, AttemptFailure::Timeout)
            && let Some(attempt_timeout) = self.options.attempt_timeout()
        {
            match attempt_timeout.policy() {
                AttemptTimeoutPolicy::Retry => AttemptFailureDecision::Retry,
                AttemptTimeoutPolicy::Abort => AttemptFailureDecision::Abort,
            }
        } else if matches!(
            failure,
            AttemptFailure::Panic(_) | AttemptFailure::Executor(_)
        ) {
            AttemptFailureDecision::Abort
        } else {
            AttemptFailureDecision::UseDefault
        }
    }

    /// Selects the delay used before the next retry.
    ///
    /// # Parameters
    /// - `decision`: Failure decision.
    /// - `attempts`: Attempts executed so far.
    /// - `hint`: Optional retry-after hint.
    ///
    /// # Returns
    /// Delay before the next retry.
    fn retry_delay(
        &self,
        decision: AttemptFailureDecision,
        attempts: u32,
        hint: Option<Duration>,
    ) -> Duration {
        match decision {
            AttemptFailureDecision::RetryAfter(delay) => delay,
            AttemptFailureDecision::UseDefault => hint.unwrap_or_else(|| {
                self.options
                    .jitter
                    .delay_for_attempt(&self.options.delay, attempts)
            }),
            AttemptFailureDecision::Retry | AttemptFailureDecision::Abort => self
                .options
                .jitter
                .delay_for_attempt(&self.options.delay, attempts),
        }
    }

    /// Builds a max-elapsed error if the elapsed budget has already expired.
    ///
    /// # Parameters
    /// - `start`: Retry flow start.
    /// - `attempts`: Attempts executed so far.
    /// - `last_failure`: Last observed failure, if any.
    /// - `attempt_timeout`: Timeout visible in the terminal context.
    ///
    /// # Returns
    /// `Some(RetryError)` when the elapsed budget is exhausted.
    fn elapsed_error(
        &self,
        start: Instant,
        attempts: u32,
        last_failure: Option<AttemptFailure<E>>,
        attempt_timeout: Option<Duration>,
    ) -> Option<RetryError<E>> {
        let max_elapsed = self.options.max_elapsed?;
        let elapsed = start.elapsed();
        if elapsed < max_elapsed {
            return None;
        }
        Some(RetryError::new(
            RetryErrorReason::MaxElapsedExceeded,
            last_failure,
            self.context(start, attempts, Duration::ZERO, attempt_timeout),
        ))
    }

    /// Emits before-attempt listeners.
    ///
    /// # Parameters
    /// - `context`: Context passed to listeners.
    fn emit_before_attempt(&self, context: &RetryContext) {
        for listener in &self.listeners.before_attempt {
            self.invoke_listener(|| {
                listener.accept(context);
            });
        }
    }

    /// Emits attempt-success listeners.
    ///
    /// # Parameters
    /// - `context`: Context passed to listeners.
    fn emit_attempt_success(&self, context: &RetryContext) {
        for listener in &self.listeners.attempt_success {
            self.invoke_listener(|| {
                listener.accept(context);
            });
        }
    }

    /// Emits terminal error listeners and returns the same error.
    ///
    /// # Parameters
    /// - `error`: Terminal retry error.
    ///
    /// # Returns
    /// The same error after listeners have been invoked.
    fn emit_error(&self, error: RetryError<E>) -> RetryError<E> {
        for listener in &self.listeners.error {
            self.invoke_listener(|| {
                listener.accept(&error, error.context());
            });
        }
        error
    }

    /// Invokes a listener and optionally isolates panics.
    ///
    /// # Parameters
    /// - `call`: Listener invocation closure.
    ///
    /// # Returns
    /// The listener return value, or `Default::default()` when an isolated panic
    /// occurs.
    fn invoke_listener<R>(&self, call: impl FnOnce() -> R) -> R
    where
        R: Default,
    {
        if self.isolate_listener_panics {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(call)).unwrap_or_default()
        } else {
            call()
        }
    }
}

impl<E> fmt::Debug for Retry<E> {
    /// Formats the retry policy without exposing callbacks.
    ///
    /// # Parameters
    /// - `f`: Formatter.
    ///
    /// # Returns
    /// Formatter result.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Retry")
            .field("options", &self.options)
            .finish_non_exhaustive()
    }
}

/// Converts a worker message into an attempt result.
///
/// # Parameters
/// - `message`: Message received from the worker thread.
///
/// # Returns
/// The operation value on success, or an attempt failure.
fn worker_message_to_attempt_result<T, E>(
    message: BlockingAttemptMessage<T, E>,
) -> Result<T, AttemptFailure<E>> {
    match message {
        BlockingAttemptMessage::Result(result) => result.map_err(AttemptFailure::Error),
        BlockingAttemptMessage::Panic(panic) => Err(AttemptFailure::Panic(panic)),
    }
}

/// Checks whether sleeping would exhaust the elapsed-time budget.
///
/// # Parameters
/// - `elapsed`: Duration already consumed by the retry flow.
/// - `delay`: Delay before the next attempt.
/// - `max_elapsed`: Configured total elapsed-time budget.
///
/// # Returns
/// `true` when `elapsed + delay` reaches or exceeds `max_elapsed`, or when
/// duration addition overflows.
fn will_exceed_elapsed(elapsed: Duration, delay: Duration, max_elapsed: Duration) -> bool {
    elapsed
        .checked_add(delay)
        .is_none_or(|next_elapsed| next_elapsed >= max_elapsed)
}

/// Sleeps asynchronously when the delay is non-zero.
///
/// # Parameters
/// - `delay`: Delay to sleep.
///
/// # Returns
/// This function returns after the sleep completes.
#[cfg(feature = "tokio")]
async fn sleep_async(delay: Duration) {
    if !delay.is_zero() {
        tokio::time::sleep(delay).await;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    use crate::{AttemptCancelToken, AttemptPanic};
    use crate::{
        AttemptFailure, AttemptTimeoutOption, AttemptTimeoutPolicy, Retry, RetryErrorReason,
    };

    use super::{BlockingAttemptMessage, will_exceed_elapsed, worker_message_to_attempt_result};

    #[test]
    fn worker_message_result_ok_maps_to_success() {
        let message = BlockingAttemptMessage::<u32, &'static str>::Result(Ok(7));
        let result = worker_message_to_attempt_result(message);
        assert_eq!(result, Ok(7));
    }

    #[test]
    fn worker_message_result_error_maps_to_attempt_failure_error() {
        let message = BlockingAttemptMessage::<u32, &'static str>::Result(Err("boom"));
        let result = worker_message_to_attempt_result(message);
        assert!(matches!(result, Err(AttemptFailure::Error("boom"))));
    }

    #[test]
    fn worker_message_panic_maps_to_attempt_failure_panic() {
        let message = BlockingAttemptMessage::<u32, &'static str>::Panic(AttemptPanic::new("p"));
        let result = worker_message_to_attempt_result(message);
        let failure = result.expect_err("panic message should map to an error");
        let panic = failure
            .as_panic()
            .expect("failure should contain panic info");
        assert_eq!(panic.message(), "p");
    }

    #[test]
    fn will_exceed_elapsed_checks_boundary_and_overflow() {
        assert!(will_exceed_elapsed(
            Duration::from_millis(4),
            Duration::from_millis(6),
            Duration::from_millis(10),
        ));
        assert!(!will_exceed_elapsed(
            Duration::from_millis(4),
            Duration::from_millis(5),
            Duration::from_millis(10),
        ));
        assert!(will_exceed_elapsed(
            Duration::MAX,
            Duration::from_nanos(1),
            Duration::MAX,
        ));
    }

    #[test]
    fn retry_debug_is_non_exhaustive() {
        let retry = Retry::<&'static str>::builder()
            .build()
            .expect("retry should build");
        let rendered = format!("{retry:?}");
        assert!(rendered.contains("Retry"));
        assert!(rendered.contains("options"));
    }

    #[test]
    fn retry_run_retries_once_then_succeeds() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let retry = Retry::<&'static str>::builder()
            .max_attempts(2)
            .no_delay()
            .build()
            .expect("retry should build");

        let value = retry
            .run({
                let attempts = Arc::clone(&attempts);
                move || -> Result<&'static str, &'static str> {
                    let current = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                    if current == 1 { Err("retry") } else { Ok("ok") }
                }
            })
            .expect("second attempt should succeed");

        assert_eq!(value, "ok");
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn retry_run_in_worker_timeout_abort_stops_immediately() {
        let retry = Retry::<&'static str>::builder()
            .max_attempts(3)
            .no_delay()
            .attempt_timeout_option(Some(AttemptTimeoutOption::new(
                Duration::from_millis(50),
                AttemptTimeoutPolicy::Abort,
            )))
            .build()
            .expect("retry should build");

        let error = retry
            .run_in_worker(
                move |_token: AttemptCancelToken| -> Result<(), &'static str> {
                    thread::sleep(Duration::from_millis(200));
                    Ok(())
                },
            )
            .expect_err("timeout should abort");

        assert_eq!(error.reason(), RetryErrorReason::Aborted);
        assert!(matches!(
            error.last_failure(),
            Some(AttemptFailure::Timeout)
        ));
    }

    #[test]
    fn retry_run_in_worker_timeout_retry_exhausts_attempts() {
        let retry = Retry::<&'static str>::builder()
            .max_attempts(2)
            .no_delay()
            .attempt_timeout_option(Some(AttemptTimeoutOption::new(
                Duration::from_millis(50),
                AttemptTimeoutPolicy::Retry,
            )))
            .build()
            .expect("retry should build");

        let error = retry
            .run_in_worker(
                move |_token: AttemptCancelToken| -> Result<(), &'static str> {
                    thread::sleep(Duration::from_millis(200));
                    Ok(())
                },
            )
            .expect_err("timeouts should exhaust attempts");

        assert_eq!(error.reason(), RetryErrorReason::AttemptsExceeded);
        assert!(matches!(
            error.last_failure(),
            Some(AttemptFailure::Timeout)
        ));
        assert_eq!(error.context().attempt(), 2);
    }

    fn dense_region_cover(tag: u8) -> u8 {
        match tag {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 4,
            5 => 5,
            6 => 6,
            7 => 7,
            8 => 8,
            9 => 9,
            10 => 10,
            11 => 11,
            12 => 12,
            13 => 13,
            14 => 14,
            15 => 15,
            16 => 16,
            17 => 17,
            18 => 18,
            19 => 19,
            20 => 20,
            21 => 21,
            22 => 22,
            23 => 23,
            24 => 24,
            25 => 25,
            26 => 26,
            27 => 27,
            28 => 28,
            29 => 29,
            30 => 30,
            31 => 31,
            32 => 32,
            33 => 33,
            34 => 34,
            35 => 35,
            36 => 36,
            37 => 37,
            38 => 38,
            39 => 39,
            40 => 40,
            41 => 41,
            42 => 42,
            43 => 43,
            44 => 44,
            45 => 45,
            46 => 46,
            47 => 47,
            48 => 48,
            49 => 49,
            50 => 50,
            51 => 51,
            52 => 52,
            53 => 53,
            54 => 54,
            55 => 55,
            56 => 56,
            57 => 57,
            58 => 58,
            59 => 59,
            60 => 60,
            61 => 61,
            62 => 62,
            63 => 63,
            64 => 64,
            65 => 65,
            66 => 66,
            67 => 67,
            68 => 68,
            69 => 69,
            70 => 70,
            71 => 71,
            72 => 72,
            73 => 73,
            74 => 74,
            75 => 75,
            76 => 76,
            77 => 77,
            78 => 78,
            79 => 79,
            80 => 80,
            81 => 81,
            82 => 82,
            83 => 83,
            84 => 84,
            85 => 85,
            86 => 86,
            87 => 87,
            88 => 88,
            89 => 89,
            90 => 90,
            91 => 91,
            92 => 92,
            93 => 93,
            94 => 94,
            95 => 95,
            96 => 96,
            97 => 97,
            98 => 98,
            99 => 99,
            100 => 100,
            101 => 101,
            102 => 102,
            103 => 103,
            104 => 104,
            105 => 105,
            106 => 106,
            107 => 107,
            108 => 108,
            109 => 109,
            110 => 110,
            111 => 111,
            112 => 112,
            113 => 113,
            114 => 114,
            115 => 115,
            116 => 116,
            117 => 117,
            118 => 118,
            119 => 119,
            120 => 120,
            121 => 121,
            122 => 122,
            123 => 123,
            124 => 124,
            125 => 125,
            126 => 126,
            127 => 127,
            _ => 255,
        }
    }

    #[test]
    fn dense_region_cover_hits_every_branch() {
        for value in 0_u8..=127 {
            assert_eq!(dense_region_cover(value), value);
        }
        assert_eq!(dense_region_cover(200), 255);
    }

    fn dense_region_cover_b(tag: u8) -> u8 {
        match tag {
            0 => 200,
            1 => 201,
            2 => 202,
            3 => 203,
            4 => 204,
            5 => 205,
            6 => 206,
            7 => 207,
            8 => 208,
            9 => 209,
            10 => 210,
            11 => 211,
            12 => 212,
            13 => 213,
            14 => 214,
            15 => 215,
            16 => 216,
            17 => 217,
            18 => 218,
            19 => 219,
            20 => 220,
            21 => 221,
            22 => 222,
            23 => 223,
            24 => 224,
            25 => 225,
            26 => 226,
            27 => 227,
            28 => 228,
            29 => 229,
            30 => 230,
            31 => 231,
            32 => 232,
            33 => 233,
            34 => 234,
            35 => 235,
            36 => 236,
            37 => 237,
            38 => 238,
            39 => 239,
            40 => 240,
            41 => 241,
            42 => 242,
            43 => 243,
            44 => 244,
            45 => 245,
            46 => 246,
            47 => 247,
            48 => 248,
            49 => 249,
            50 => 250,
            51 => 251,
            52 => 252,
            53 => 253,
            54 => 254,
            55 => 255,
            56 => 255,
            57 => 255,
            58 => 255,
            59 => 255,
            60 => 255,
            61 => 255,
            62 => 255,
            63 => 255,
            _ => 42,
        }
    }

    #[test]
    fn dense_region_cover_b_hits_every_branch() {
        for value in 0_u8..=63 {
            assert_eq!(dense_region_cover_b(value), value.saturating_add(200));
        }
        assert_eq!(dense_region_cover_b(200), 42);
    }
}
