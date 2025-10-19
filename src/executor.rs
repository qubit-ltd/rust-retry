/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Retry Executor
//!
//! Retry executor responsible for executing operations with retry strategies.
//!
//! # Author
//!
//! Haixing Hu

use std::error::Error;
use std::time::{Duration, Instant};

use prism3_function::readonly_consumer::ReadonlyConsumer;

use super::{
    AbortEvent, DefaultRetryConfig, FailureEvent, RetryBuilder, RetryConfig, RetryDecision,
    RetryError, RetryEvent, RetryReason, RetryResult, SuccessEvent,
};

/// Retry executor
///
/// Responsible for executing operations with retry strategies. Automatically
/// executes retry logic according to configured retry strategies, delay
/// strategies, failure/abort conditions, and triggers event listeners at
/// appropriate times.
///
/// # Generic Parameters
///
/// * `T` - The return value type of the operation
/// * `C` - Retry configuration type, must implement `RetryConfig` trait,
///   defaults to `DefaultRetryConfig`
///
/// # Core Features
///
/// - **Synchronous Retry**: `run()` method executes synchronous operations,
///   using post-check mechanism for timeout detection
/// - **Asynchronous Retry**: `run_async()` method executes asynchronous
///   operations, using tokio::time::timeout for real timeout interruption
/// - **Timeout Control**: Supports single operation timeout (operation_timeout)
///   and overall timeout (max_duration)
/// - **Event Listening**: Supports event callbacks for retry, success,
///   failure, abort, etc.
/// - **Flexible Configuration**: Supports multiple delay strategies, error
///   type identification, result value judgment, etc.
///
/// # Timeout Control
///
/// The executor supports two levels of timeout control:
///
/// 1. **Single Operation Timeout (operation_timeout)**:
///    - Controls the maximum execution time for each operation
///    - Synchronous version (`run`): Checks if timeout occurred after
///      operation completes (post-check mechanism)
///    - Asynchronous version (`run_async`): Uses tokio::time::timeout to
///      truly interrupt timeout operations
///
/// 2. **Overall Timeout (max_duration)**:
///    - Controls the maximum total time for the entire retry process
///      (including all retries and delays)
///    - Applies to both synchronous and asynchronous versions
///
/// # Usage Examples
///
/// ## Synchronous Retry (Post-Check Timeout)
///
/// ```rust
/// use prism3_retry::{RetryBuilder, RetryResult};
/// use std::time::Duration;
///
/// let executor = RetryBuilder::<String>::new()
///     .set_max_attempts(3)
///     .set_operation_timeout(Some(Duration::from_secs(5)))
///     .build();
///
/// // Use RetryResult type alias to simplify function signature
/// let result: RetryResult<String> = executor.run(|| {
///     // Can directly return any error type that implements Into<RetryError>
///     // For example, using ? operator to handle io::Error will automatically
///     // convert to RetryError
///     std::thread::sleep(Duration::from_millis(100));
///     Ok("SUCCESS".to_string())
/// });
/// ```
///
/// ## Asynchronous Retry (Real Timeout Interruption)
///
/// ```rust,no_run
/// use prism3_retry::{RetryBuilder, RetryResult};
/// use std::time::Duration;
///
/// # async fn example() {
/// let executor = RetryBuilder::<String>::new()
///     .set_max_attempts(3)
///     .set_operation_timeout(Some(Duration::from_secs(5)))
///     .build();
///
/// // Use RetryResult to make async function signature clearer
/// let result: RetryResult<String> = executor.run_async(|| async {
///     // Asynchronous operation, truly interrupted on timeout
///     tokio::time::sleep(Duration::from_millis(100)).await;
///     Ok("SUCCESS".to_string())
/// }).await;
/// # }
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct RetryExecutor<T, C: RetryConfig = DefaultRetryConfig> {
    builder: RetryBuilder<T, C>,
}

impl<T, C> RetryExecutor<T, C>
where
    T: Clone + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static,
    C: RetryConfig,
{
    /// Create retry executor
    pub(crate) fn new(builder: RetryBuilder<T, C>) -> Self {
        Self { builder }
    }

    // ==================== Private Helper Methods ====================

    /// Check if maximum duration has been exceeded
    ///
    /// # Parameters
    ///
    /// * `start_time` - Start time
    /// * `max_duration` - Maximum duration
    /// * `attempt` - Current attempt count
    ///
    /// # Returns
    ///
    /// Returns Some(RetryError) if maximum duration exceeded, None otherwise
    fn check_max_duration_exceeded(
        &self,
        start_time: Instant,
        max_duration: Option<Duration>,
        attempt: u32,
    ) -> Option<RetryError> {
        if let Some(max_dur) = max_duration {
            let elapsed = start_time.elapsed();
            if elapsed >= max_dur {
                let failure_event = FailureEvent::builder()
                    .attempt_count(attempt)
                    .total_duration(elapsed)
                    .build();
                if let Some(listener) = self.builder.failure_listener() {
                    listener.accept(&failure_event);
                }
                return Some(RetryError::max_duration_exceeded(elapsed, max_dur));
            }
        }
        None
    }

    /// Check if single operation timeout occurred (post-check mechanism)
    ///
    /// # Parameters
    ///
    /// * `result` - Operation result
    /// * `operation_duration` - Operation execution time
    ///
    /// # Returns
    ///
    /// Returns timeout error if timed out, otherwise returns original result
    fn check_operation_timeout(
        &self,
        result: Result<T, Box<dyn Error + Send + Sync>>,
        operation_duration: Duration,
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
        if let Some(timeout) = self.builder.operation_timeout() {
            if operation_duration > timeout {
                return Err(
                    Box::new(RetryError::operation_timeout(operation_duration, timeout))
                        as Box<dyn Error + Send + Sync>,
                );
            }
        }
        result
    }

    /// Handle success case
    ///
    /// # Parameters
    ///
    /// * `value` - Successful result value
    /// * `attempt` - Current attempt count
    /// * `start_time` - Start time
    ///
    /// # Returns
    ///
    /// Returns success result
    fn handle_success(&self, value: T, attempt: u32, start_time: Instant) -> RetryResult<T> {
        let success_event = SuccessEvent::builder()
            .result(value.clone())
            .attempt_count(attempt)
            .total_duration(start_time.elapsed())
            .build();
        if let Some(listener) = self.builder.success_listener() {
            listener.accept(&success_event);
        }
        Ok(value)
    }

    /// Handle abort case
    ///
    /// # Parameters
    ///
    /// * `reason` - Abort reason
    /// * `attempt` - Current attempt count
    /// * `start_time` - Start time
    ///
    /// # Returns
    ///
    /// Returns abort error
    fn handle_abort(
        &self,
        reason: super::AbortReason<T>,
        attempt: u32,
        start_time: Instant,
    ) -> RetryResult<T> {
        let abort_event = AbortEvent::builder()
            .reason(reason)
            .attempt_count(attempt)
            .total_duration(start_time.elapsed())
            .build();
        if let Some(listener) = self.builder.abort_listener() {
            listener.accept(&abort_event);
        }
        Err(RetryError::aborted("Operation aborted"))
    }

    /// Check if maximum attempts reached
    ///
    /// # Parameters
    ///
    /// * `attempt` - Current attempt count
    /// * `max_attempts` - Maximum attempts
    ///
    /// # Returns
    ///
    /// Returns true if maximum attempts reached, false otherwise
    fn check_max_attempts_exceeded(&self, attempt: u32, max_attempts: u32) -> bool {
        attempt >= max_attempts
    }

    /// Handle maximum attempts exceeded case
    ///
    /// # Parameters
    ///
    /// * `attempt` - Current attempt count
    /// * `max_attempts` - Maximum attempts
    /// * `reason` - Retry reason
    /// * `start_time` - Start time
    ///
    /// # Returns
    ///
    /// Returns maximum attempts exceeded error
    fn handle_max_attempts_exceeded(
        &self,
        attempt: u32,
        max_attempts: u32,
        reason: RetryReason<T>,
        start_time: Instant,
    ) -> RetryError {
        let failure_event = match reason {
            RetryReason::Error(error) => FailureEvent::builder()
                .last_error(Some(error))
                .attempt_count(attempt)
                .total_duration(start_time.elapsed())
                .build(),
            RetryReason::Result(result) => FailureEvent::builder()
                .last_result(Some(result))
                .attempt_count(attempt)
                .total_duration(start_time.elapsed())
                .build(),
        };

        if let Some(listener) = self.builder.failure_listener() {
            listener.accept(&failure_event);
        }

        RetryError::max_attempts_exceeded(attempt, max_attempts)
    }

    /// Calculate delay duration
    ///
    /// # Parameters
    ///
    /// * `attempt` - Current attempt count
    ///
    /// # Returns
    ///
    /// Returns calculated delay duration
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_strategy = self.builder.delay_strategy();
        let jitter_factor = self.builder.jitter_factor();
        delay_strategy.calculate_delay(attempt, jitter_factor)
    }

    /// Create retry event
    ///
    /// # Parameters
    ///
    /// * `attempt` - Current attempt count
    /// * `max_attempts` - Maximum attempts
    /// * `reason` - Retry reason
    /// * `delay` - Delay duration
    /// * `start_time` - Start time
    ///
    /// # Returns
    ///
    /// Returns created retry event
    fn create_retry_event(
        &self,
        attempt: u32,
        max_attempts: u32,
        reason: RetryReason<T>,
        delay: Duration,
        start_time: Instant,
    ) -> RetryEvent<T> {
        match reason {
            RetryReason::Error(error) => RetryEvent::builder()
                .attempt_count(attempt)
                .max_attempts(max_attempts)
                .last_error(Some(error))
                .next_delay(delay)
                .total_duration(start_time.elapsed())
                .build(),
            RetryReason::Result(result) => RetryEvent::builder()
                .attempt_count(attempt)
                .max_attempts(max_attempts)
                .last_result(Some(result))
                .next_delay(delay)
                .total_duration(start_time.elapsed())
                .build(),
        }
    }

    /// Trigger retry event and wait for delay
    ///
    /// # Parameters
    ///
    /// * `retry_event` - Retry event
    /// * `delay` - Delay duration
    fn trigger_retry_and_wait(&self, retry_event: RetryEvent<T>, delay: Duration) {
        if let Some(listener) = self.builder.retry_listener() {
            listener.accept(&retry_event);
        }

        if delay > Duration::ZERO {
            std::thread::sleep(delay);
        }
    }

    /// Trigger retry event and wait for delay asynchronously
    ///
    /// # Parameters
    ///
    /// * `retry_event` - Retry event
    /// * `delay` - Delay duration
    async fn trigger_retry_and_wait_async(&self, retry_event: RetryEvent<T>, delay: Duration) {
        if let Some(listener) = self.builder.retry_listener() {
            listener.accept(&retry_event);
        }

        if delay > Duration::ZERO {
            tokio::time::sleep(delay).await;
        }
    }

    /// Execute single synchronous operation and get decision
    ///
    /// # Parameters
    ///
    /// * `operation` - Operation to execute
    ///
    /// # Returns
    ///
    /// Returns retry decision
    fn execute_operation_and_get_decision<F>(&self, operation: &mut F) -> RetryDecision<T>
    where
        F: FnMut() -> Result<T, Box<dyn Error + Send + Sync>>,
    {
        let operation_start = Instant::now();
        let result = operation();
        let operation_duration = operation_start.elapsed();

        // Check single operation timeout (post-check mechanism)
        let result = self.check_operation_timeout(result, operation_duration);

        // Get retry decision
        self.builder.get_retry_decision(result)
    }

    /// Execute single asynchronous operation and get decision
    ///
    /// # Parameters
    ///
    /// * `operation` - Asynchronous operation to execute
    ///
    /// # Returns
    ///
    /// Returns retry decision
    async fn execute_operation_async_and_get_decision<F, Fut>(
        &self,
        operation: &mut F,
    ) -> RetryDecision<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        let operation_start = Instant::now();
        let operation_timeout = self.builder.operation_timeout();

        let result = if let Some(timeout_duration) = operation_timeout {
            // With timeout limit, use tokio::time::timeout
            match tokio::time::timeout(timeout_duration, operation()).await {
                Ok(result) => result,
                Err(_elapsed) => {
                    // Timed out, convert to error
                    let duration = operation_start.elapsed();
                    Err(
                        Box::new(RetryError::operation_timeout(duration, timeout_duration))
                            as Box<dyn Error + Send + Sync>,
                    )
                }
            }
        } else {
            // No timeout limit, execute directly
            operation().await
        };

        // Get retry decision
        self.builder.get_retry_decision(result)
    }

    /// Handle retry decision and return whether to continue
    ///
    /// # Parameters
    ///
    /// * `decision` - Retry decision
    /// * `attempt` - Current attempt count
    /// * `max_attempts` - Maximum attempts
    /// * `start_time` - Start time
    ///
    /// # Returns
    ///
    /// - `Ok(Some(value))` - Operation succeeded, returns result value
    /// - `Ok(None)` - Need to retry, returns None to continue loop
    /// - `Err(error)` - Operation failed or aborted, returns error
    fn handle_decision(
        &self,
        decision: RetryDecision<T>,
        attempt: u32,
        max_attempts: u32,
        start_time: Instant,
    ) -> Result<Option<T>, RetryError> {
        match decision {
            RetryDecision::Success(value) => {
                self.handle_success(value.clone(), attempt, start_time)?;
                Ok(Some(value))
            }
            RetryDecision::Retry(reason) => {
                // Check if maximum retry count reached
                if self.check_max_attempts_exceeded(attempt, max_attempts) {
                    let error = self.handle_max_attempts_exceeded(
                        attempt,
                        max_attempts,
                        reason,
                        start_time,
                    );
                    return Err(error);
                }

                // Calculate delay and create retry event
                let delay = self.calculate_delay(attempt);
                let retry_event =
                    self.create_retry_event(attempt, max_attempts, reason, delay, start_time);

                // Return None and delay time to indicate retry needed
                // Note: We need to return delay time, so need to adjust return type
                // Or trigger event directly here
                self.trigger_retry_and_wait(retry_event, delay);

                Ok(None) // Return None to indicate need to continue retrying
            }
            RetryDecision::Abort(reason) => {
                self.handle_abort(reason, attempt, start_time).map(|_| None) // Won't reach here as handle_abort always returns Err
            }
        }
    }

    /// Handle async retry decision and return whether to continue
    ///
    /// # Parameters
    ///
    /// * `decision` - Retry decision
    /// * `attempt` - Current attempt count
    /// * `max_attempts` - Maximum attempts
    /// * `start_time` - Start time
    ///
    /// # Returns
    ///
    /// - `Ok(Some(value))` - Operation succeeded, returns result value
    /// - `Ok(None)` - Need to retry, returns None to continue loop
    /// - `Err(error)` - Operation failed or aborted, returns error
    async fn handle_decision_async(
        &self,
        decision: RetryDecision<T>,
        attempt: u32,
        max_attempts: u32,
        start_time: Instant,
    ) -> Result<Option<T>, RetryError> {
        match decision {
            RetryDecision::Success(value) => {
                self.handle_success(value.clone(), attempt, start_time)?;
                Ok(Some(value))
            }
            RetryDecision::Retry(reason) => {
                // Check if maximum retry count reached
                if self.check_max_attempts_exceeded(attempt, max_attempts) {
                    let error = self.handle_max_attempts_exceeded(
                        attempt,
                        max_attempts,
                        reason,
                        start_time,
                    );
                    return Err(error);
                }

                // Calculate delay and create retry event
                let delay = self.calculate_delay(attempt);
                let retry_event =
                    self.create_retry_event(attempt, max_attempts, reason, delay, start_time);

                // Trigger event and wait asynchronously
                self.trigger_retry_and_wait_async(retry_event, delay).await;

                Ok(None) // Return None to indicate need to continue retrying
            }
            RetryDecision::Abort(reason) => {
                self.handle_abort(reason, attempt, start_time).map(|_| None) // Won't reach here as handle_abort always returns Err
            }
        }
    }

    // ==================== Public Methods ====================

    /// Execute synchronous operation (with post-check timeout mechanism)
    ///
    /// Execute synchronous operation according to configured retry strategy,
    /// until success, maximum retry count reached, or abort condition met.
    ///
    /// # Timeout Control
    ///
    /// This method uses **post-check mechanism** for timeout control:
    /// - After operation completes, check if execution time exceeds
    ///   `operation_timeout`
    /// - If timeout, convert result to `RetryError::OperationTimeout` error
    ///   and trigger retry
    /// - Note: Cannot truly interrupt ongoing synchronous operation
    ///
    /// If you need to truly interrupt timeout operations, please use
    /// `run_async()` method.
    ///
    /// # Parameters
    ///
    /// * `operation` - Operation to execute, returns
    ///   `Result<T, Box<dyn Error + Send + Sync>>`
    ///
    /// # Returns
    ///
    /// Returns operation result or error
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_retry::{RetryBuilder, RetryDelayStrategy, RetryResult};
    /// use std::time::Duration;
    ///
    /// let executor = RetryBuilder::new()
    ///     .set_max_attempts(3)
    ///     .set_delay_strategy(RetryDelayStrategy::Fixed { delay: Duration::from_secs(1) })
    ///     .set_operation_timeout(Some(Duration::from_secs(5))) // Single operation post-check timeout
    ///     .build();
    ///
    /// // Use RetryResult to simplify function signature, leveraging From trait
    /// // for automatic error conversion
    /// let result: RetryResult<String> = executor.run(|| -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    ///     // Can return any standard error type, will be automatically
    ///     // converted to RetryError
    ///     // Example: std::fs::File::open("file.txt")?;
    ///     // io::Error will be automatically converted to RetryError through
    ///     // From trait
    ///     Ok("SUCCESS".to_string())
    /// });
    ///
    /// assert!(result.is_ok());
    /// ```
    pub fn run<F>(&self, mut operation: F) -> RetryResult<T>
    where
        F: FnMut() -> Result<T, Box<dyn Error + Send + Sync>>,
    {
        let start_time = Instant::now();
        let max_attempts = self.builder.max_attempts();
        let max_duration = self.builder.max_duration();
        let mut attempt = 0;

        loop {
            attempt += 1;

            // Check if maximum duration exceeded
            if let Some(error) = self.check_max_duration_exceeded(start_time, max_duration, attempt)
            {
                return Err(error);
            }

            // Execute operation and get decision
            let decision = self.execute_operation_and_get_decision(&mut operation);

            // Handle decision
            match self.handle_decision(decision, attempt, max_attempts, start_time)? {
                Some(value) => return Ok(value), // Success, return result
                None => continue,                // Retry, continue to next iteration
            }
        }
    }

    /// Execute asynchronous operation (with real timeout interruption)
    ///
    /// Execute asynchronous operation according to configured retry strategy,
    /// with single operation timeout control.
    ///
    /// # Timeout Control
    ///
    /// This method uses **tokio::time::timeout** for real timeout interruption:
    /// - When operation execution time exceeds `operation_timeout`, the
    ///   operation will be truly interrupted (cancelled)
    /// - After interruption, retry will be triggered (if there are remaining
    ///   retry attempts)
    /// - Compared to the `run()` method's post-check mechanism, this approach
    ///   is more efficient and precise
    ///
    /// # Difference from Synchronous Version
    ///
    /// | Feature | `run()` Sync Version | `run_async()` Async Version |
    /// |---------|---------------------|----------------------------|
    /// | Timeout Mechanism | Post-check (check after operation completes) | Real interruption (tokio::time::timeout) |
    /// | Can Interrupt Operation | ❌ Cannot | ✅ Can |
    /// | Timeout Precision | Depends on operation completion | Precise to millisecond level |
    /// | Applicable Scenario | Short synchronous operations | Long asynchronous operations |
    ///
    /// # Parameters
    ///
    /// * `operation` - Asynchronous operation to execute
    ///
    /// # Returns
    ///
    /// Returns operation result or error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use prism3_retry::{RetryBuilder, RetryDelayStrategy, RetryResult};
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let executor = RetryBuilder::<String>::new()
    ///         .set_max_attempts(3)
    ///         .set_operation_timeout(Some(Duration::from_secs(5)))  // Real timeout interruption
    ///         .set_max_duration(Some(Duration::from_secs(30)))      // Overall timeout
    ///         .set_delay_strategy(RetryDelayStrategy::Fixed {
    ///             delay: Duration::from_secs(1)
    ///         })
    ///         .build();
    ///
    ///     // Use RetryResult type alias to make code more concise
    ///     let result: RetryResult<String> = executor.run_async(|| async {
    ///         // Can also use ? operator in async operations, errors will be
    ///         // automatically converted
    ///         // Example: tokio::fs::read_to_string("file.txt").await?;
    ///         tokio::time::sleep(Duration::from_millis(100)).await;
    ///         Ok("SUCCESS".to_string())
    ///     }).await;
    ///
    ///     assert!(result.is_ok());
    /// }
    /// ```
    pub async fn run_async<F, Fut>(&self, mut operation: F) -> RetryResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        let start_time = Instant::now();
        let max_attempts = self.builder.max_attempts();
        let max_duration = self.builder.max_duration();
        let mut attempt = 0;

        loop {
            attempt += 1;

            // Check if maximum duration exceeded
            if let Some(error) = self.check_max_duration_exceeded(start_time, max_duration, attempt)
            {
                return Err(error);
            }

            // Execute operation and get decision
            let decision = self
                .execute_operation_async_and_get_decision(&mut operation)
                .await;

            // Handle decision
            match self
                .handle_decision_async(decision, attempt, max_attempts, start_time)
                .await?
            {
                Some(value) => return Ok(value), // Success, return result
                None => continue,                // Retry, continue to next iteration
            }
        }
    }

    /// Get builder configuration
    pub fn config(&self) -> &RetryBuilder<T, C> {
        &self.builder
    }
}
