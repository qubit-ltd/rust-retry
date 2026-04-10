/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Retry Builder
//!
//! Builder for retry executor, used to configure and build retry strategies.
//!
//! # Author
//!
//! Haixing Hu

use std::any::TypeId;
use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;

use qubit_function::Consumer;
use qubit_function::{BoxPredicate, Predicate};

use super::event::{
    AbortEventListener, FailureEventListener, RetryEventListener, SuccessEventListener,
};
use super::{
    AbortEvent, AbortReason, DefaultRetryConfig, FailureEvent, RetryConfig, RetryDecision,
    RetryDelayStrategy, RetryEvent, RetryReason, SuccessEvent,
};

/// Condition predicate type for result evaluation
///
/// Uses `BoxPredicate` for single ownership predicate evaluation.
/// Since RetryBuilder follows the builder pattern and is designed for single-threaded use,
/// Box is more appropriate than Arc, avoiding unnecessary reference counting overhead.
type ConditionPredicate<T> = Option<BoxPredicate<T>>;

/// Retry Builder
///
/// Builder for configuring and constructing retry strategies. Provides rich configuration options, including retry strategies, delay strategies,
/// failure/abort conditions, event listeners, etc. Supports method chaining, making retry logic configuration more intuitive and flexible.
///
/// ## Generic Parameters
///
/// * `T` - The return type of the operation
/// * `C` - Retry configuration type, must implement the `RetryConfig` trait, defaults to `DefaultRetryConfig`
///
/// ## Configuration Override Semantics
///
/// **⚠️ Important: Non-cumulative Effect**
///
/// Configuration methods have **override semantics**, **not cumulative effect**:
/// - **Failure Condition Configuration**: `failed_on_*` methods clear previous failure condition configurations
/// - **Abort Condition Configuration**: `abort_on_*` methods clear previous abort condition configurations
/// - **Listener Configuration**: `on_*` methods replace previous listener configurations
///
/// ## Default Behavior for Error Handling
///
/// **Consistent with Java Failsafe default behavior:**
/// - **Retry All Errors by Default**: If no error configuration methods are called, all errors will be retried by default
/// - **Explicit Error Configuration**: Calling `failed_on_error*()` methods overrides default behavior, only retrying specified errors
/// - **Explicitly Retry All Errors**: Calling `failed_on_all_errors()` explicitly expresses the intent to retry all errors
/// - **Explicitly Disable Error Retry**: Calling `no_failed_errors()` disables all error retries
///
/// # Author
///
/// Haixing Hu
pub struct RetryBuilder<T, C: RetryConfig = DefaultRetryConfig> {
    /// Retry configuration
    config: C,

    /// Set of error type IDs that need to be retried
    failed_error_types: HashSet<TypeId>,

    /// Set of result values that need to be retried
    failed_results: HashSet<T>,

    /// Predicate for result conditions that need to be retried
    failed_condition: ConditionPredicate<T>,

    /// Set of error type IDs that need to be aborted
    abort_error_types: HashSet<TypeId>,

    /// Set of result values that need to be aborted
    abort_results: HashSet<T>,

    /// Predicate for result conditions that need to be aborted
    abort_condition: ConditionPredicate<T>,

    /// Retry event listener, triggered after each retry attempt fails
    on_retry: Option<RetryEventListener<T>>,

    /// Success event listener, triggered when operation completes successfully
    on_success: Option<SuccessEventListener<T>>,

    /// Failure event listener, triggered when operation ultimately fails after all retries
    on_failure: Option<FailureEventListener<T>>,

    /// Abort event listener, triggered when operation is aborted
    on_abort: Option<AbortEventListener<T>>,
}

// Provide new() method for default config type
impl<T> RetryBuilder<T, DefaultRetryConfig>
where
    T: Clone + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static,
{
    /// Construct a retry builder with default configuration
    ///
    /// # Returns
    ///
    /// Returns a new `RetryBuilder` instance using `DefaultRetryConfig`
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_retry::RetryBuilder;
    ///
    /// let builder = RetryBuilder::<String>::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            config: DefaultRetryConfig::new(),
            failed_error_types: HashSet::new(),
            failed_results: HashSet::new(),
            failed_condition: None,
            abort_error_types: HashSet::new(),
            abort_results: HashSet::new(),
            abort_condition: None,
            on_retry: None,
            on_success: None,
            on_failure: None,
            on_abort: None,
        }
    }
}

// Provide common methods for all configuration types
impl<T, C> RetryBuilder<T, C>
where
    T: Clone + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static,
    C: RetryConfig,
{
    /// Construct a retry builder with specified configuration
    ///
    /// # Parameters
    ///
    /// * `config` - Retry configuration instance
    ///
    /// # Returns
    ///
    /// Returns a new `RetryBuilder` instance using the specified configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_retry::{RetryBuilder, DefaultRetryConfig};
    ///
    /// let config = DefaultRetryConfig::new();
    /// let builder = RetryBuilder::<String, _>::with_config(config);
    /// ```
    #[inline]
    pub fn with_config(config: C) -> Self {
        Self {
            config,
            failed_error_types: HashSet::new(),
            failed_results: HashSet::new(),
            failed_condition: None,
            abort_error_types: HashSet::new(),
            abort_results: HashSet::new(),
            abort_condition: None,
            on_retry: None,
            on_success: None,
            on_failure: None,
            on_abort: None,
        }
    }

    // --- Basic retry control methods ---

    /// Get maximum number of attempts
    #[inline]
    pub fn max_attempts(&self) -> u32 {
        self.config.max_attempts()
    }

    /// Set maximum number of attempts
    #[inline]
    pub fn set_max_attempts(mut self, max_attempts: u32) -> Self {
        self.config.set_max_attempts(max_attempts);
        self
    }

    /// Get maximum duration for executing retries
    #[inline]
    pub fn max_duration(&self) -> Option<Duration> {
        self.config.max_duration()
    }

    /// Set maximum duration for executing retries
    #[inline]
    pub fn set_max_duration(mut self, max_duration: Option<Duration>) -> Self {
        self.config.set_max_duration(max_duration);
        self
    }

    /// Get single operation timeout
    ///
    /// Single operation timeout controls the maximum time for each execution. This is different from max_duration:
    /// - operation_timeout: Maximum execution time for a single operation
    /// - max_duration: Total time for the entire retry process (including all retries and delays)
    ///
    /// # Returns
    ///
    /// Returns `Some(Duration)` if there is a timeout limit, `None` for unlimited
    #[inline]
    pub fn operation_timeout(&self) -> Option<Duration> {
        self.config.operation_timeout()
    }

    /// Set single operation timeout
    ///
    /// Set timeout limit for each operation. If a single operation execution time exceeds this time,
    /// it will be considered a failure and trigger retry (if there are remaining retry attempts).
    ///
    /// # Parameters
    ///
    /// * `timeout` - Timeout duration, `Some(Duration)` for limited, `None` for unlimited
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_retry::RetryBuilder;
    /// use std::time::Duration;
    ///
    /// let executor = RetryBuilder::<String>::new()
    ///     .set_max_attempts(3)
    ///     .set_operation_timeout(Some(Duration::from_secs(5))) // Single operation max 5 seconds
    ///     .set_max_duration(Some(Duration::from_secs(30)))     // Total time max 30 seconds
    ///     .build();
    /// ```
    #[inline]
    pub fn set_operation_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.config.set_operation_timeout(timeout);
        self
    }

    /// Set unlimited operation timeout
    ///
    /// This is a convenience method that sets single operation timeout to None, indicating no time limit for single operations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_retry::RetryBuilder;
    ///
    /// let executor = RetryBuilder::<String>::new()
    ///     .set_unlimited_operation_timeout()
    ///     .build();
    /// ```
    #[inline]
    pub fn set_unlimited_operation_timeout(mut self) -> Self {
        self.config.set_unlimited_operation_timeout();
        self
    }

    // --- Delay strategy methods ---

    /// Get delay strategy type
    #[inline]
    pub fn delay_strategy(&self) -> RetryDelayStrategy {
        self.config.delay_strategy()
    }

    /// Set delay strategy type
    #[inline]
    pub fn set_delay_strategy(mut self, delay_strategy: RetryDelayStrategy) -> Self {
        self.config.set_delay_strategy(delay_strategy);
        self
    }

    /// Get jitter factor
    #[inline]
    pub fn jitter_factor(&self) -> f64 {
        self.config.jitter_factor()
    }

    /// Set jitter factor
    #[inline]
    pub fn set_jitter_factor(mut self, jitter_factor: f64) -> Self {
        self.config.set_jitter_factor(jitter_factor);
        self
    }

    // --- Convenience delay strategy methods ---

    /// Set random delay range
    #[inline]
    pub fn set_random_delay_strategy(mut self, min_delay: Duration, max_delay: Duration) -> Self {
        self.config.set_random_delay_strategy(min_delay, max_delay);
        self
    }

    /// Set fixed delay
    #[inline]
    pub fn set_fixed_delay_strategy(mut self, delay: Duration) -> Self {
        self.config.set_fixed_delay_strategy(delay);
        self
    }

    /// Set exponential backoff strategy parameters
    #[inline]
    pub fn set_exponential_backoff_strategy(
        mut self,
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
    ) -> Self {
        self.config
            .set_exponential_backoff_strategy(initial_delay, max_delay, multiplier);
        self
    }

    /// Set no delay strategy
    #[inline]
    pub fn set_no_delay_strategy(mut self) -> Self {
        self.config.set_no_delay_strategy();
        self
    }

    /// Set unlimited duration
    #[inline]
    pub fn set_unlimited_duration(mut self) -> Self {
        self.config.set_unlimited_duration();
        self
    }

    // --- Failure condition configuration ---

    /// Explicitly disable error retry
    ///
    /// After calling this method, no errors will be treated as failure conditions, all errors will be returned directly without triggering retry.
    /// This overrides the default behavior of retrying all errors.
    #[inline]
    pub fn no_failed_errors(mut self) -> Self {
        self.failed_error_types.clear();
        // Add a non-existent error type to override default behavior
        self.failed_error_types
            .insert(TypeId::of::<NonExistentError>());
        self
    }

    /// Explicitly configure retry for all errors
    ///
    /// Calling this method is equivalent to calling `failed_on_error::<std::error::Error>()`, will retry all errors.
    /// Although this is also the default behavior, this method makes the user's intent more explicit.
    #[inline]
    pub fn failed_on_all_errors(mut self) -> Self {
        self.failed_error_types.clear();
        self.failed_error_types.insert(TypeId::of::<dyn Error>());
        self
    }

    /// Set error types that indicate failure
    ///
    /// When the executed code returns these types of errors, it will be considered a failure, and will trigger retry if the retry count has not been exceeded;
    /// otherwise it will terminate retry and return the error.
    ///
    /// **⚠️ Important: Override Semantics**
    /// This method will **clear** all failure errors set through other `failed_on_error` methods,
    /// and then set new failure errors. **No cumulative effect.**
    #[inline]
    pub fn failed_on_error<E: Error + 'static>(mut self) -> Self {
        self.failed_error_types.clear();
        self.failed_error_types.insert(TypeId::of::<E>());
        self
    }

    /// Set error types that indicate failure (multiple)
    ///
    /// **⚠️ Important: Override Semantics**
    /// This method will **clear** all failure errors set through other `failed_on_error` methods,
    /// and then set new failure error list. **No cumulative effect.**
    #[inline]
    pub fn failed_on_errors<E1: Error + 'static, E2: Error + 'static>(mut self) -> Self {
        self.failed_error_types.clear();
        self.failed_error_types.insert(TypeId::of::<E1>());
        self.failed_error_types.insert(TypeId::of::<E2>());
        self
    }

    /// Set result that indicates failure
    ///
    /// If the return value of the operation equals the specified failure result, the operation is considered failed, and will trigger retry if the retry count has not been exceeded;
    /// otherwise it will terminate retry. Result comparison uses the `PartialEq` trait.
    ///
    /// **⚠️ Important: Override Semantics**
    /// This method will **clear** all failure results set through other `failed_on_result` methods,
    /// and then set a new single failure result. **No cumulative effect.**
    #[inline]
    pub fn failed_on_result(mut self, result: T) -> Self {
        self.failed_results.clear();
        self.failed_results.insert(result);
        self
    }

    /// Set results that indicate failure (multiple)
    ///
    /// **⚠️ Important: Override Semantics**
    /// This method will **clear** all failure results set through other `failed_on_result` methods,
    /// and then set new multiple failure results. **No cumulative effect.**
    #[inline]
    pub fn failed_on_results(mut self, results: Vec<T>) -> Self {
        self.failed_results.clear();
        self.failed_results.extend(results);
        self
    }

    /// Set condition for determining failure results
    ///
    /// If the return value of the operation meets the failure result determination condition, the operation is considered failed and will trigger retry.
    ///
    /// **⚠️ Important: Override Semantics**
    /// This method will **replace** the failure condition set through `failed_on_results_if`.
    /// **No cumulative effect.**
    ///
    /// # Implementation Note
    ///
    /// Accepts any closure implementing `Fn(&T) -> bool`.
    /// The closure is internally converted to `BoxPredicate` for storage.
    #[inline]
    pub fn failed_on_results_if<P>(mut self, condition: P) -> Self
    where
        P: Fn(&T) -> bool + 'static,
    {
        self.failed_condition = Some(condition.into_box());
        self
    }

    /// Clear all failure results
    #[inline]
    pub fn clear_failed_results(mut self) -> Self {
        self.failed_results.clear();
        self
    }

    // --- Abort condition configuration ---

    /// Set error types that need to terminate retry
    ///
    /// When the executed code returns these types of errors, it will immediately terminate retry and trigger abort logic.
    ///
    /// **⚠️ Important: Override Semantics**
    /// This method will **clear** all abort errors set through other `abort_on_error` methods,
    /// and then set new abort errors. **No cumulative effect.**
    #[inline]
    pub fn abort_on_error<E: Error + 'static>(mut self) -> Self {
        self.abort_error_types.clear();
        self.abort_error_types.insert(TypeId::of::<E>());
        self
    }

    /// Set error types that need to terminate retry (multiple)
    #[inline]
    pub fn abort_on_errors<E1: Error + 'static, E2: Error + 'static>(mut self) -> Self {
        self.abort_error_types.clear();
        self.abort_error_types.insert(TypeId::of::<E1>());
        self.abort_error_types.insert(TypeId::of::<E2>());
        self
    }

    /// Explicitly configure abort for all errors
    ///
    /// Calling this method will abort on all errors, immediately stopping the retry process.
    /// This is useful when you want to fail fast on any error without retrying.
    ///
    /// **⚠️ Important: Override Semantics**
    /// This method will **clear** all abort errors set through other `abort_on_error` methods,
    /// and then configure to abort on all error types. **No cumulative effect.**
    #[inline]
    pub fn abort_on_all_errors(mut self) -> Self {
        self.abort_error_types.clear();
        self.abort_error_types.insert(TypeId::of::<dyn Error>());
        self
    }

    /// Set result that needs to abort retry
    ///
    /// If the return value of the operation equals the specified abort result, the operation should abort retry and will not continue trying.
    /// Abort is different from failure: failure triggers retry, while abort immediately stops the retry process.
    ///
    /// **⚠️ Important: Override Semantics**
    /// This method will **clear** all abort results set through other `abort_on_result` methods,
    /// and then set a new single abort result. **No cumulative effect.**
    #[inline]
    pub fn abort_on_result(mut self, result: T) -> Self {
        self.abort_results.clear();
        self.abort_results.insert(result);
        self
    }

    /// Set results that need to abort retry (multiple)
    #[inline]
    pub fn abort_on_results(mut self, results: Vec<T>) -> Self {
        self.abort_results.clear();
        self.abort_results.extend(results);
        self
    }

    /// Set condition for determining abort results
    ///
    /// If the return value of the operation meets the abort result determination condition, the operation should abort retry and will not continue trying.
    ///
    /// **⚠️ Important: Override Semantics**
    /// This method will **replace** the abort condition set through `abort_on_results_if`.
    /// **No cumulative effect.**
    ///
    /// # Implementation Note
    ///
    /// Accepts any closure implementing `Fn(&T) -> bool`.
    /// The closure is internally converted to `BoxPredicate` for storage.
    #[inline]
    pub fn abort_on_results_if<P>(mut self, condition: P) -> Self
    where
        P: Fn(&T) -> bool + 'static,
    {
        self.abort_condition = Some(condition.into_box());
        self
    }

    /// Clear all abort results
    #[inline]
    pub fn clear_abort_results(mut self) -> Self {
        self.abort_results.clear();
        self
    }

    // --- Event listener configuration ---

    /// Set retry event listener
    ///
    /// The retry event listener is called after each retry attempt fails. Through this listener you can:
    /// - Log detailed retry information
    /// - Track retry count and failure reasons
    /// - Send alert notifications
    /// - Execute custom retry logic
    ///
    /// # Lifetime Requirements
    ///
    /// ⚠️ **Important**: The listener must be self-contained and not
    /// depend on local variables from the calling scope. The listener will
    /// be stored in the `RetryExecutor` and may be invoked long after the
    /// builder is created.
    ///
    /// **Valid approaches**:
    /// - Use closures that don't capture any variables
    /// - Capture `'static` data (e.g., `static` variables)
    /// - Capture `Arc`-wrapped data that can outlive the current scope
    /// - Implement a struct with `Consumer` trait
    ///
    /// **Invalid approach** (won't compile):
    /// ```rust,compile_fail
    /// # use qubit_retry::RetryBuilder;
    /// # use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    /// let local_var = Arc::new(AtomicBool::new(false));
    /// let local_clone = local_var.clone();
    /// let builder = RetryBuilder::<i32>::new().on_retry(move |_event| {
    ///     local_clone.store(true, Ordering::SeqCst); // ❌ Won't compile
    /// });
    /// ```
    ///
    /// **Valid approach** (using struct):
    /// ```rust
    /// # use qubit_retry::RetryBuilder;
    /// # use qubit_function::Consumer;
    /// # use qubit_retry::RetryEvent;
    /// # use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
    /// struct MyListener {
    ///     counter: Arc<AtomicUsize>,
    /// }
    ///
    /// impl Consumer<RetryEvent<i32>> for MyListener {
    ///     fn accept(&self, _event: &RetryEvent<i32>) {
    ///         self.counter.fetch_add(1, Ordering::SeqCst);
    ///     }
    /// }
    ///
    /// let counter = Arc::new(AtomicUsize::new(0));
    /// let listener = MyListener { counter: counter.clone() };
    /// let builder = RetryBuilder::<i32>::new().on_retry(listener);
    /// ```
    ///
    /// # Implementation Note
    ///
    /// Accepts any type implementing `Consumer<RetryEvent<T>>`
    /// trait, including closures. Closures automatically implement
    /// `Consumer` trait, so you can pass them directly.
    #[inline]
    pub fn on_retry<L>(mut self, listener: L) -> Self
    where
        L: Consumer<RetryEvent<T>> + 'static,
    {
        self.on_retry = Some(listener.into_box());
        self
    }

    /// Set success event listener
    ///
    /// The success event listener is triggered when the operation completes
    /// successfully, whether it succeeds on the first attempt or after
    /// retries.
    ///
    /// # Lifetime Requirements
    ///
    /// ⚠️ **Important**: The listener must be self-contained and not
    /// depend on local variables from the calling scope. The listener will
    /// be stored in the `RetryExecutor` and may be invoked long after the
    /// builder is created.
    ///
    /// **Valid approaches**:
    /// - Use closures that don't capture any variables
    /// - Capture `'static` data (e.g., `static` variables)
    /// - Capture `Arc`-wrapped data that can outlive the current scope
    /// - Implement a struct with `Consumer` trait
    ///
    /// See [`on_retry`](Self::on_retry) for detailed examples.
    ///
    /// # Implementation Note
    ///
    /// Accepts any type implementing `Consumer<SuccessEvent<T>>`
    /// trait, including closures. Closures automatically implement
    /// `Consumer` trait, so you can pass them directly.
    #[inline]
    pub fn on_success<L>(mut self, listener: L) -> Self
    where
        L: Consumer<SuccessEvent<T>> + 'static,
    {
        self.on_success = Some(listener.into_box());
        self
    }

    /// Set failure event listener
    ///
    /// The failure event listener is triggered when the operation
    /// ultimately fails, i.e., after all retry attempts have failed.
    ///
    /// # Lifetime Requirements
    ///
    /// ⚠️ **Important**: The listener must be self-contained and not
    /// depend on local variables from the calling scope. The listener will
    /// be stored in the `RetryExecutor` and may be invoked long after the
    /// builder is created.
    ///
    /// **Valid approaches**:
    /// - Use closures that don't capture any variables
    /// - Capture `'static` data (e.g., `static` variables)
    /// - Capture `Arc`-wrapped data that can outlive the current scope
    /// - Implement a struct with `Consumer` trait
    ///
    /// See [`on_retry`](Self::on_retry) for detailed examples.
    ///
    /// # Implementation Note
    ///
    /// Accepts any type implementing `Consumer<FailureEvent<T>>`
    /// trait, including closures. Closures automatically implement
    /// `Consumer` trait, so you can pass them directly.
    #[inline]
    pub fn on_failure<L>(mut self, listener: L) -> Self
    where
        L: Consumer<FailureEvent<T>> + 'static,
    {
        self.on_failure = Some(listener.into_box());
        self
    }

    /// Set abort event listener
    ///
    /// The abort event listener is triggered when the operation is aborted.
    ///
    /// **⚠️ Key: Abort Listener Trigger Conditions**
    /// Whether the abort listener is triggered depends on **whether the
    /// result is also defined as "failed"**:
    /// - **✅ Will trigger listener**: When the result matching abort
    ///   condition **also matches failure condition**
    /// - **❌ Will not trigger listener**: When the result matching abort
    ///   condition **does not match any failure condition**
    ///
    /// # Lifetime Requirements
    ///
    /// ⚠️ **Important**: The listener must be self-contained and not
    /// depend on local variables from the calling scope. The listener will
    /// be stored in the `RetryExecutor` and may be invoked long after the
    /// builder is created.
    ///
    /// **Valid approaches**:
    /// - Use closures that don't capture any variables
    /// - Capture `'static` data (e.g., `static` variables)
    /// - Capture `Arc`-wrapped data that can outlive the current scope
    /// - Implement a struct with `Consumer` trait
    ///
    /// See [`on_retry`](Self::on_retry) for detailed examples.
    ///
    /// # Implementation Note
    ///
    /// Accepts any type implementing `Consumer<AbortEvent<T>>`
    /// trait, including closures. Closures automatically implement
    /// `Consumer` trait, so you can pass them directly.
    #[inline]
    pub fn on_abort<L>(mut self, listener: L) -> Self
    where
        L: Consumer<AbortEvent<T>> + 'static,
    {
        self.on_abort = Some(listener.into_box());
        self
    }

    // --- Build method ---

    /// Build retry executor
    ///
    /// # Returns
    ///
    /// Returns a configured `RetryExecutor` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_retry::RetryBuilder;
    ///
    /// let executor = RetryBuilder::<String>::new()
    ///     .set_max_attempts(3)
    ///     .build();
    /// ```
    #[inline]
    pub fn build(self) -> super::executor::RetryExecutor<T, C> {
        super::executor::RetryExecutor::new(self)
    }

    // --- Internal methods ---

    /// Determine if error should be retried
    pub(crate) fn should_retry_error(&self, _error: &(dyn Error + 'static)) -> bool {
        // If specific error types are configured, check if they match
        if !self.failed_error_types.is_empty() {
            // Check if it is dyn Error type (indicates retry all errors)
            if self.failed_error_types.contains(&TypeId::of::<dyn Error>()) {
                return true;
            }

            // Check if it is non-existent error type (indicates disable error retry)
            if self
                .failed_error_types
                .contains(&TypeId::of::<NonExistentError>())
            {
                return false;
            }

            // For other specific error types, we need to check via downcast
            // Here we use a simpler approach: if specific error types are configured,
            // we assume the user wants to retry errors of these types
            return true;
        }

        // Default behavior: retry all errors
        true
    }

    /// Determine if error should be aborted
    pub(crate) fn should_abort_error(&self, _error: &(dyn Error + 'static)) -> bool {
        // For abort errors, we adopt a similar strategy
        if !self.abort_error_types.is_empty() {
            // Check if it is dyn Error type (indicates abort all errors)
            if self.abort_error_types.contains(&TypeId::of::<dyn Error>()) {
                return true;
            }

            // For other specific error types, we assume the user wants to abort errors of these types
            return true;
        }

        false
    }

    /// Determine if result should be retried
    pub(crate) fn should_retry_result(&self, result: &T) -> bool {
        // Check specific result values
        if self.failed_results.contains(result) {
            return true;
        }

        // Check result condition
        if let Some(ref condition) = self.failed_condition {
            if condition.test(result) {
                return true;
            }
        }

        false
    }

    /// Determine if result should be aborted
    pub(crate) fn should_abort_result(&self, result: &T) -> bool {
        // Check specific result values
        if self.abort_results.contains(result) {
            return true;
        }

        // Check result condition
        if let Some(ref condition) = self.abort_condition {
            if condition.test(result) {
                return true;
            }
        }

        false
    }

    /// Get retry decision
    pub(crate) fn get_retry_decision(
        &self,
        result: Result<T, Box<dyn Error + Send + Sync>>,
    ) -> RetryDecision<T> {
        match result {
            Ok(value) => {
                // Check if result needs to be retried
                if self.should_retry_result(&value) {
                    RetryDecision::Retry(RetryReason::Result(value))
                } else if self.should_abort_result(&value) {
                    RetryDecision::Abort(AbortReason::Result(value))
                } else {
                    RetryDecision::Success(value)
                }
            }
            Err(error) => {
                // Check error type
                if self.should_abort_error(error.as_ref()) {
                    RetryDecision::Abort(AbortReason::Error(error))
                } else if self.should_retry_error(error.as_ref()) {
                    RetryDecision::Retry(RetryReason::Error(error))
                } else {
                    // Neither retry nor abort, return error directly
                    RetryDecision::Abort(AbortReason::Error(error))
                }
            }
        }
    }

    /// Get event listeners
    #[inline]
    pub(crate) fn retry_listener(&self) -> &Option<RetryEventListener<T>> {
        &self.on_retry
    }

    #[inline]
    pub(crate) fn success_listener(&self) -> &Option<SuccessEventListener<T>> {
        &self.on_success
    }

    #[inline]
    pub(crate) fn failure_listener(&self) -> &Option<FailureEventListener<T>> {
        &self.on_failure
    }

    #[inline]
    pub(crate) fn abort_listener(&self) -> &Option<AbortEventListener<T>> {
        &self.on_abort
    }
}

impl<T> Default for RetryBuilder<T, DefaultRetryConfig>
where
    T: Clone + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// A non-existent error type, used to override default behavior
///
/// This is a sentinel type used only for its `TypeId` to mark the "disable error retry" state.
/// It is never instantiated, so its `Display` implementation will never be called.
#[derive(Debug)]
struct NonExistentError;

impl std::fmt::Display for NonExistentError {
    // NOTE: This method is required by the Error trait but will never be called in practice.
    // NonExistentError is a sentinel type that is never instantiated - it's only used for
    // its TypeId to mark the "disable error retry" state in the type system.
    // The 3 uncovered lines here (including the function signature and body) are expected
    // and documented as unreachable code that exists only to satisfy trait bounds.
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Error for NonExistentError {}
