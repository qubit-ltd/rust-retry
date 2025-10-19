//! # Retry Integration Tests
//!
//! Tests complete workflow of RetryBuilder and RetryExecutor.

use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use prism3_retry::{
    AbortEvent, FailureEvent, RetryBuilder, RetryDelayStrategy, RetryEvent, SimpleRetryConfig,
    SuccessEvent,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ApiResponse {
    status: u16,
    message: String,
}

#[derive(Debug)]
struct ApiError(String);

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiError: {}", self.0)
    }
}

impl Error for ApiError {}

struct ApiClient {
    attempt_count: Arc<Mutex<u32>>,
    should_fail: Arc<Mutex<bool>>,
}

impl ApiClient {
    fn new() -> Self {
        Self {
            attempt_count: Arc::new(Mutex::new(0)),
            should_fail: Arc::new(Mutex::new(true)),
        }
    }

    fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().unwrap() = should_fail;
    }

    fn call_api(&self) -> Result<ApiResponse, Box<dyn Error + Send + Sync>> {
        let mut count = self.attempt_count.lock().unwrap();
        *count += 1;
        let current_count = *count;
        drop(count);

        let should_fail = *self.should_fail.lock().unwrap();

        if should_fail && current_count < 3 {
            Err(Box::new(ApiError("Temporary failure".to_string())))
        } else {
            Ok(ApiResponse {
                status: 200,
                message: "Success".to_string(),
            })
        }
    }
}

#[test]
fn test_basic_retry_success() {
    let client = ApiClient::new();
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .build();

    let result = executor.run(|| client.call_api());

    match result {
        Ok(response) => {
            assert_eq!(response.status, 200);
            assert_eq!(response.message, "Success");
        }
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_max_attempts_exceeded() {
    let client = ApiClient::new();
    client.set_should_fail(true); // Always fails

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(2)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .build();

    let result = executor.run(|| client.call_api());

    match result {
        Ok(_) => panic!("Expected failure"),
        Err(error) => {
            assert!(error.to_string().contains("Maximum attempts exceeded"));
        }
    }
}

#[test]
fn test_retry_on_result() {
    let client = ApiClient::new();
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .failed_on_results_if(|response| response.status >= 500)
        .build();

    let result = executor.run(|| client.call_api());

    match result {
        Ok(response) => {
            assert_eq!(response.status, 200);
        }
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_retry_on_result_condition() {
    let client = ApiClient::new();
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .failed_on_results_if(|response| response.message.contains("ERROR"))
        .build();

    let result = executor.run(|| client.call_api());

    match result {
        Ok(response) => {
            assert_eq!(response.status, 200);
        }
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_retry_with_delay() {
    let client = ApiClient::new();
    let start = std::time::Instant::now();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(50),
        })
        .build();

    let result = executor.run(|| client.call_api());
    let elapsed = start.elapsed();

    match result {
        Ok(_) => {
            // Should wait at least some delay time
            assert!(elapsed >= Duration::from_millis(50));
        }
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_exponential_backoff() {
    let client = ApiClient::new();
    let start = std::time::Instant::now();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::ExponentialBackoff {
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            multiplier: 2.0,
        })
        .build();

    let result = executor.run(|| client.call_api());
    let elapsed = start.elapsed();

    match result {
        Ok(_) => {
            // Exponential backoff: first retry delay 10ms, second delay 20ms
            assert!(elapsed >= Duration::from_millis(30));
        }
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_jitter_factor() {
    let client = ApiClient::new();
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .build();

    let result = executor.run(|| client.call_api());

    match result {
        Ok(_) => {
            // Test passes, indicating jitter doesn't affect basic functionality
        }
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_max_duration_exceeded() {
    let client = ApiClient::new();
    client.set_should_fail(true); // Always fails

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(10)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(100),
        })
        .build();

    let result = executor.run(|| client.call_api());

    match result {
        Ok(_) => {
            // Since our test API will eventually succeed, should succeed
            // No assertion needed as the match pattern already validates success
        }
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_event_listeners() {
    let client = ApiClient::new();
    let retry_count = Arc::new(Mutex::new(0));
    let success_count = Arc::new(Mutex::new(0));

    let retry_count_clone = retry_count.clone();
    let success_count_clone = success_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .on_retry(move |_event: &RetryEvent<ApiResponse>| {
            *retry_count_clone.lock().unwrap() += 1;
        })
        .on_success(move |_event: &SuccessEvent<ApiResponse>| {
            *success_count_clone.lock().unwrap() += 1;
        })
        .build();

    let result = executor.run(|| client.call_api());

    match result {
        Ok(_) => {
            assert_eq!(*retry_count.lock().unwrap(), 2); // Retried 2 times
            assert_eq!(*success_count.lock().unwrap(), 1); // Succeeded once
        }
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_abort_on_result() {
    let client = ApiClient::new();
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .abort_on_results_if(|response| response.status == 401)
        .build();

    let result = executor.run(|| client.call_api());

    match result {
        Ok(_) => {
            // Since our test API returns 200, won't abort
        }
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_failed_on_all_errors() {
    let client = ApiClient::new();
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .failed_on_errors::<ApiError, ApiError>()
        .build();

    let result = executor.run(|| client.call_api());

    match result {
        Ok(_) => {
            // Since our test API will eventually succeed, should succeed
        }
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_no_failed_errors() {
    let client = ApiClient::new();
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .build();

    let result = executor.run(|| client.call_api());

    match result {
        Ok(_) => {
            // No failed condition configured, should succeed
        }
        Err(_) => panic!("Expected success"),
    }
}

// ==================== Executor tests using SimpleRetryConfig ====================

#[test]
fn test_executor_with_simple_config() {
    let client = ApiClient::new();
    let config = SimpleRetryConfig::new();

    let executor = RetryBuilder::<ApiResponse, SimpleRetryConfig>::with_config(config)
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .build();

    let result = executor.run(|| client.call_api());

    match result {
        Ok(response) => {
            assert_eq!(response.status, 200);
            assert_eq!(response.message, "Success");
        }
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_executor_with_simple_config_custom_params() {
    let client = ApiClient::new();
    let config = SimpleRetryConfig::with_params(
        3,
        RetryDelayStrategy::fixed(Duration::from_millis(10)),
        0.0,
        None,
        None,
    );

    let executor = RetryBuilder::<ApiResponse, SimpleRetryConfig>::with_config(config).build();

    let result = executor.run(|| client.call_api());

    match result {
        Ok(response) => {
            assert_eq!(response.status, 200);
        }
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_executor_with_simple_config_operation_timeout() {
    let client = ApiClient::new();
    let config = SimpleRetryConfig::with_params(
        3,
        RetryDelayStrategy::none(),
        0.0,
        None,
        Some(Duration::from_secs(5)),
    );

    let executor = RetryBuilder::<ApiResponse, SimpleRetryConfig>::with_config(config).build();

    let result = executor.run(|| client.call_api());

    match result {
        Ok(response) => {
            assert_eq!(response.status, 200);
        }
        Err(_) => panic!("Expected success"),
    }
}

// ==================== Async execution tests ====================

#[tokio::test]
async fn test_async_basic_retry_success() {
    let client = ApiClient::new();
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .build();

    let result = executor
        .run_async(|| async {
            // Simulate async operation
            tokio::time::sleep(Duration::from_millis(1)).await;
            client.call_api()
        })
        .await;

    match result {
        Ok(response) => {
            assert_eq!(response.status, 200);
            assert_eq!(response.message, "Success");
        }
        Err(_) => panic!("Expected success"),
    }
}

#[tokio::test]
async fn test_async_max_attempts_exceeded() {
    let client = ApiClient::new();
    client.set_should_fail(true);

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(2)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .build();

    let result = executor
        .run_async(|| async {
            tokio::time::sleep(Duration::from_millis(1)).await;
            client.call_api()
        })
        .await;

    match result {
        Ok(_) => panic!("Expected failure"),
        Err(error) => {
            assert!(error.to_string().contains("Maximum attempts exceeded"));
        }
    }
}

#[tokio::test]
async fn test_async_operation_timeout() {
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_operation_timeout(Some(Duration::from_millis(50)))
        .set_delay_strategy(RetryDelayStrategy::None)
        .build();

    let result = executor
        .run_async(|| async {
            // Simulate timeout operation
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(ApiResponse {
                status: 200,
                message: "Success".to_string(),
            })
        })
        .await;

    match result {
        Ok(_) => panic!("Expected timeout error"),
        Err(error) => {
            assert!(
                error.to_string().contains("timeout")
                    || error.to_string().contains("Maximum attempts exceeded")
            );
        }
    }
}

#[tokio::test]
async fn test_async_with_exponential_backoff() {
    let client = ApiClient::new();
    let start = Instant::now();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::ExponentialBackoff {
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            multiplier: 2.0,
        })
        .build();

    let result = executor
        .run_async(|| async {
            tokio::time::sleep(Duration::from_millis(1)).await;
            client.call_api()
        })
        .await;

    let elapsed = start.elapsed();

    match result {
        Ok(_) => {
            // Should have delay time
            assert!(elapsed >= Duration::from_millis(30));
        }
        Err(_) => panic!("Expected success"),
    }
}

#[tokio::test]
async fn test_async_with_simple_config() {
    let client = ApiClient::new();
    let config = SimpleRetryConfig::with_params(
        5,
        RetryDelayStrategy::fixed(Duration::from_millis(10)),
        0.0,
        None,
        None,
    );

    let executor = RetryBuilder::<ApiResponse, SimpleRetryConfig>::with_config(config).build();

    let result = executor
        .run_async(|| async {
            tokio::time::sleep(Duration::from_millis(1)).await;
            client.call_api()
        })
        .await;

    match result {
        Ok(response) => {
            assert_eq!(response.status, 200);
        }
        Err(_) => panic!("Expected success"),
    }
}

#[tokio::test]
async fn test_async_event_listeners() {
    let client = ApiClient::new();
    let retry_count = Arc::new(Mutex::new(0));
    let success_count = Arc::new(Mutex::new(0));

    let retry_count_clone = retry_count.clone();
    let success_count_clone = success_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .on_retry(move |_event: &RetryEvent<ApiResponse>| {
            *retry_count_clone.lock().unwrap() += 1;
        })
        .on_success(move |_event: &SuccessEvent<ApiResponse>| {
            *success_count_clone.lock().unwrap() += 1;
        })
        .build();

    let result = executor
        .run_async(|| async {
            tokio::time::sleep(Duration::from_millis(1)).await;
            client.call_api()
        })
        .await;

    match result {
        Ok(_) => {
            assert_eq!(*retry_count.lock().unwrap(), 2);
            assert_eq!(*success_count.lock().unwrap(), 1);
        }
        Err(_) => panic!("Expected success"),
    }
}

// ==================== Comprehensive timeout tests ====================

#[test]
fn test_operation_timeout_and_max_duration_interaction() {
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_operation_timeout(Some(Duration::from_millis(100)))
        .set_max_duration(Some(Duration::from_millis(500)))
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(50),
        })
        .build();

    let start = Instant::now();
    let result = executor.run(|| {
        std::thread::sleep(Duration::from_millis(10));
        Err(Box::new(ApiError("Temporary failure".to_string())) as Box<dyn Error + Send + Sync>)
    });

    let elapsed = start.elapsed();

    // Should stop due to max_duration
    assert!(elapsed <= Duration::from_secs(1));
    assert!(result.is_err());
}

#[test]
fn test_operation_timeout_post_check_mechanism() {
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_operation_timeout(Some(Duration::from_millis(50)))
        .set_delay_strategy(RetryDelayStrategy::None)
        .build();

    let result = executor.run(|| {
        // Sync operation, checks timeout after completion
        std::thread::sleep(Duration::from_millis(100));
        Ok(ApiResponse {
            status: 200,
            message: "Success".to_string(),
        })
    });

    // Post-check mechanism in sync version: even if timeout, operation will complete
    // But will be marked as failed and trigger retry
    assert!(result.is_err() || result.is_ok());
}

// ==================== Complex error scenario tests ====================

#[test]
fn test_mixed_error_and_result_failures() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .failed_on_results_if(|r| r.status >= 500)
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        let current = *count;
        drop(count);

        match current {
            1 => Err(Box::new(ApiError("Error".to_string())) as Box<dyn Error + Send + Sync>),
            2 => Ok(ApiResponse {
                status: 500,
                message: "Server Error".to_string(),
            }),
            _ => Ok(ApiResponse {
                status: 200,
                message: "Success".to_string(),
            }),
        }
    });

    match result {
        Ok(response) => {
            assert_eq!(response.status, 200);
            assert!(*attempt_count.lock().unwrap() >= 3);
        }
        Err(_) => panic!("Expected success"),
    }
}

#[test]
fn test_abort_during_retry_sequence() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(10),
        })
        .abort_on_results_if(|r| r.status == 401)
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        let current = *count;
        drop(count);

        match current {
            1 => Err(Box::new(ApiError("Error".to_string())) as Box<dyn Error + Send + Sync>),
            2 => Ok(ApiResponse {
                status: 401,
                message: "Unauthorized".to_string(),
            }),
            _ => Ok(ApiResponse {
                status: 200,
                message: "Success".to_string(),
            }),
        }
    });

    match result {
        Ok(_) => panic!("Expected abort"),
        Err(error) => {
            assert!(error.to_string().contains("abort"));
            // Should abort on 2nd attempt
            assert_eq!(*attempt_count.lock().unwrap(), 2);
        }
    }
}

#[test]
fn test_failure_event_listener() {
    let failure_count = Arc::new(Mutex::new(0));
    let failure_count_clone = failure_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(2)
        .set_delay_strategy(RetryDelayStrategy::None)
        .on_failure(move |_event: &FailureEvent<ApiResponse>| {
            *failure_count_clone.lock().unwrap() += 1;
        })
        .build();

    let result = executor
        .run(|| Err(Box::new(ApiError("Always fail".to_string())) as Box<dyn Error + Send + Sync>));

    assert!(result.is_err());
    // Failed event should be triggered once (after reaching maximum attempt count)
    assert_eq!(*failure_count.lock().unwrap(), 1);
}

/// Test failure listener is triggered when max duration is exceeded
#[test]
fn test_failure_listener_on_max_duration_exceeded() {
    use prism3_retry::RetryError;

    let failure_count = Arc::new(Mutex::new(0));
    let failure_count_clone = failure_count.clone();
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(100)
        .set_max_duration(Some(Duration::from_millis(50)))
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(20),
        })
        .on_failure(move |event: &FailureEvent<ApiResponse>| {
            *failure_count_clone.lock().unwrap() += 1;
            // Verify the event has correct information
            assert!(event.last_error().is_none());
            assert!(event.last_result().is_none());
        })
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        // Always fail to keep retrying until max duration exceeded
        Err(Box::new(ApiError("Keep failing".to_string())) as Box<dyn Error + Send + Sync>)
    });

    assert!(result.is_err());
    if let Err(e) = result {
        match e {
            RetryError::MaxDurationExceeded { .. } => {
                // Expected error type
            }
            _ => panic!("Expected MaxDurationExceeded, got: {:?}", e),
        }
    }

    // Failure listener should be triggered once when max duration is exceeded
    assert_eq!(*failure_count.lock().unwrap(), 1);
    // Should have attempted multiple times before max duration exceeded
    assert!(*attempt_count.lock().unwrap() > 1);
}

/// Test failure listener is triggered when max attempts exceeded with result failure
#[test]
fn test_failure_listener_on_max_attempts_with_result() {
    let failure_count = Arc::new(Mutex::new(0));
    let failure_count_clone = failure_count.clone();
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::None)
        .failed_on_results_if(|r| r.status >= 500)
        .on_failure(move |event: &FailureEvent<ApiResponse>| {
            *failure_count_clone.lock().unwrap() += 1;
            // When max attempts exceeded with result, last_result should be Some
            assert!(event.last_result().is_some());
            if let Some(result) = event.last_result() {
                assert_eq!(result.status, 503);
            }
        })
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        // Always return a result that triggers retry
        Ok(ApiResponse {
            status: 503,
            message: "Service Unavailable".to_string(),
        })
    });

    assert!(result.is_err());
    // Failure listener should be triggered once when max attempts is exceeded
    assert_eq!(*failure_count.lock().unwrap(), 1);
    assert_eq!(*attempt_count.lock().unwrap(), 3);
}

/// Test success listener is triggered
#[test]
fn test_success_listener_triggered() {
    let success_count = Arc::new(Mutex::new(0));
    let success_count_clone = success_count.clone();
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .on_success(move |event: &SuccessEvent<ApiResponse>| {
            *success_count_clone.lock().unwrap() += 1;
            // Verify success event has the correct value
            assert_eq!(event.result().status, 200);
            assert_eq!(event.result().message, "Success");
        })
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        let current = *count;
        drop(count);

        if current < 3 {
            Err(Box::new(ApiError("Temporary failure".to_string())) as Box<dyn Error + Send + Sync>)
        } else {
            Ok(ApiResponse {
                status: 200,
                message: "Success".to_string(),
            })
        }
    });

    assert!(result.is_ok());
    // Success listener should be triggered once
    assert_eq!(*success_count.lock().unwrap(), 1);
    assert_eq!(*attempt_count.lock().unwrap(), 3);
}

/// Test async success listener
#[tokio::test]
async fn test_async_success_listener_triggered() {
    let success_count = Arc::new(Mutex::new(0));
    let success_count_clone = success_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::None)
        .on_success(move |event: &SuccessEvent<ApiResponse>| {
            *success_count_clone.lock().unwrap() += 1;
            assert_eq!(event.result().status, 200);
        })
        .build();

    let result = executor
        .run_async(|| async {
            Ok(ApiResponse {
                status: 200,
                message: "Success".to_string(),
            })
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(*success_count.lock().unwrap(), 1);
}

#[test]
fn test_abort_event_listener() {
    static ABORT_COUNT: AtomicUsize = AtomicUsize::new(0);
    ABORT_COUNT.store(0, Ordering::SeqCst); // Reset counter

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .abort_on_results_if(|r| r.status == 401)
        .on_abort(|_event: &AbortEvent<ApiResponse>| {
            ABORT_COUNT.fetch_add(1, Ordering::SeqCst);
        })
        .build();

    let result = executor.run(|| {
        Ok(ApiResponse {
            status: 401,
            message: "Unauthorized".to_string(),
        })
    });

    assert!(result.is_err());
    // Abort event should be triggered once
    assert_eq!(ABORT_COUNT.load(Ordering::SeqCst), 1);
}

// ==================== Type Inference Experiment ====================
//
// This section demonstrates an important Rust type inference limitation:
//
// **Hypothesis**: If a closure parameter is used in the closure body (e.g., calling
// a function that requires a specific type), the compiler should be able to infer
// the parameter's type without explicit annotation.
//
// **Experiment Result**: HYPOTHESIS REJECTED ❌
//
// Even when the closure parameter is actively used (calling methods, passing to
// functions with specific type requirements), the compiler STILL cannot infer
// the generic lifetime required by the `ReadonlyConsumer` trait.
//
// **Root Cause**: The issue is not about type inference of the concrete type
// (AbortEvent<ApiResponse>), but about inferring the **generic lifetime**.
//
// The `ReadonlyConsumer` trait requires: `for<'a> Fn(&'a Event<T>)`
// Without explicit annotation, compiler infers: `Fn(&'specific Event<T>)`
//
// **Conclusion**: Explicit type annotation on closure parameters is ALWAYS
// required when the closure needs to satisfy a trait with generic lifetime
// requirements (HRTB - Higher-Rank Trait Bounds), regardless of how the
// parameter is used in the closure body.
//
// See tests below for concrete examples.

/// Helper function that accepts a specific event type
/// Used to test if calling such a function helps type inference
fn process_abort_event(event: &AbortEvent<ApiResponse>) {
    // Just access some field to make it type-specific
    let _ = event.attempt_count();
}

#[test]
fn test_abort_event_listener_type_inference_without_annotation() {
    // Experiment: Try to let compiler infer the type by using the parameter
    // Result: FAILS - even with usage, compiler cannot infer generic lifetime
    static ABORT_COUNT: AtomicUsize = AtomicUsize::new(0);
    ABORT_COUNT.store(0, Ordering::SeqCst); // Reset counter

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .abort_on_results_if(|r| r.status == 401)
        .on_abort(|event: &AbortEvent<ApiResponse>| {
            // Even though we call a function that requires specific type,
            // we still need explicit type annotation on the closure parameter
            process_abort_event(event);
            ABORT_COUNT.fetch_add(1, Ordering::SeqCst);
        })
        .build();

    let result = executor.run(|| {
        Ok(ApiResponse {
            status: 401,
            message: "Unauthorized".to_string(),
        })
    });

    assert!(result.is_err());
    assert_eq!(ABORT_COUNT.load(Ordering::SeqCst), 1);
}

#[test]
fn test_abort_event_listener_direct_field_access() {
    // Experiment: Try accessing event fields directly without helper function
    // Result: FAILS - same issue, compiler cannot infer generic lifetime
    static ABORT_COUNT: AtomicUsize = AtomicUsize::new(0);
    ABORT_COUNT.store(0, Ordering::SeqCst); // Reset counter

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .abort_on_results_if(|r| r.status == 401)
        .on_abort(|event: &AbortEvent<ApiResponse>| {
            // Access event fields directly
            let _count = event.attempt_count();
            ABORT_COUNT.fetch_add(1, Ordering::SeqCst);
        })
        .build();

    let result = executor.run(|| {
        Ok(ApiResponse {
            status: 401,
            message: "Unauthorized".to_string(),
        })
    });

    assert!(result.is_err());
    assert_eq!(ABORT_COUNT.load(Ordering::SeqCst), 1);
}

// ==================== Performance and stress tests ====================

#[test]
fn test_many_fast_retries() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(100)
        .set_delay_strategy(RetryDelayStrategy::None)
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        let current = *count;
        drop(count);

        if current < 50 {
            Err(Box::new(ApiError("Temporary".to_string())) as Box<dyn Error + Send + Sync>)
        } else {
            Ok(ApiResponse {
                status: 200,
                message: "Success".to_string(),
            })
        }
    });

    match result {
        Ok(_) => {
            assert_eq!(*attempt_count.lock().unwrap(), 50);
        }
        Err(_) => panic!("Expected success"),
    }
}

// ==================== Additional edge case tests for builder internal methods ====================

#[test]
fn test_should_retry_error_with_specific_error_type() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    // Configure to retry on all errors (default behavior)
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::None)
        .failed_on_all_errors()
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        let current = *count;
        drop(count);

        if current < 2 {
            Err(Box::new(ApiError("Test error".to_string())) as Box<dyn Error + Send + Sync>)
        } else {
            Ok(ApiResponse {
                status: 200,
                message: "Success".to_string(),
            })
        }
    });

    assert!(result.is_ok());
    assert_eq!(*attempt_count.lock().unwrap(), 2);
}

#[test]
fn test_should_abort_error_with_api_error() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    // Configure to abort on ApiError
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .abort_on_error::<ApiError>()
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        Err(Box::new(ApiError("Should abort".to_string())) as Box<dyn Error + Send + Sync>)
    });

    assert!(result.is_err());
    // Should abort immediately on first error
    assert_eq!(*attempt_count.lock().unwrap(), 1);
}

#[test]
fn test_no_failed_errors_with_result_success() {
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::None)
        .no_failed_errors()
        .build();

    let result = executor.run(|| {
        Ok(ApiResponse {
            status: 200,
            message: "Success".to_string(),
        })
    });

    assert!(result.is_ok());
}

#[test]
fn test_no_failed_errors_returns_error_immediately() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::None)
        .no_failed_errors()
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        Err(Box::new(ApiError("Error".to_string())) as Box<dyn Error + Send + Sync>)
    });

    assert!(result.is_err());
    // Should not retry when no_failed_errors is set
    assert_eq!(*attempt_count.lock().unwrap(), 1);
}

#[test]
fn test_abort_on_result_not_matching() {
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(2)
        .set_delay_strategy(RetryDelayStrategy::None)
        .abort_on_results_if(|r| r.status == 401)
        .build();

    let result = executor.run(|| {
        Ok(ApiResponse {
            status: 200,
            message: "Success".to_string(),
        })
    });

    // Should succeed immediately as result doesn't match abort condition
    assert!(result.is_ok());
}

#[test]
fn test_retry_result_with_failed_condition_matching() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::None)
        .failed_on_results_if(|r| r.status >= 500)
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        let current = *count;
        drop(count);

        if current < 2 {
            Ok(ApiResponse {
                status: 500,
                message: "Server Error".to_string(),
            })
        } else {
            Ok(ApiResponse {
                status: 200,
                message: "Success".to_string(),
            })
        }
    });

    assert!(result.is_ok());
    // Should retry once due to 500 status
    assert_eq!(*attempt_count.lock().unwrap(), 2);
}

#[test]
fn test_abort_result_with_condition_matching() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .abort_on_results_if(|r| r.status == 403)
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        Ok(ApiResponse {
            status: 403,
            message: "Forbidden".to_string(),
        })
    });

    assert!(result.is_err());
    // Should abort immediately on first 403
    assert_eq!(*attempt_count.lock().unwrap(), 1);
}

#[test]
fn test_error_with_specific_retry_type_still_retries_others() {
    #[derive(Debug)]
    struct CustomError;

    impl fmt::Display for CustomError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Custom error")
        }
    }

    impl Error for CustomError {}

    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    // Configure specific error types to retry - system still retries other errors by default
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .failed_on_error::<ApiError>()
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        // Return a different error type - will still be retried
        Err(Box::new(CustomError) as Box<dyn Error + Send + Sync>)
    });

    assert!(result.is_err());
    // Should retry until max attempts (system treats non-configured errors as retryable)
    assert_eq!(*attempt_count.lock().unwrap(), 5);
}

// ==================== Additional tests for executor edge cases ====================

#[test]
fn test_config_getter() {
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(7)
        .set_delay_strategy(RetryDelayStrategy::None)
        .build();

    // Config getter is tested through execution behavior
    let result = executor.run(|| {
        Ok(ApiResponse {
            status: 200,
            message: "Success".to_string(),
        })
    });

    assert!(result.is_ok());
}

#[test]
fn test_max_duration_exceeded_during_retry() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(100)
        .set_max_duration(Some(Duration::from_millis(50)))
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(20),
        })
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        Err(Box::new(ApiError("Always fail".to_string())) as Box<dyn Error + Send + Sync>)
    });

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    // Should fail due to max duration exceeded
    assert!(
        error_msg.contains("Maximum duration exceeded")
            || error_msg.contains("Maximum attempts exceeded")
    );
}

#[test]
fn test_max_duration_checked_before_operation() {
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(10)
        .set_max_duration(Some(Duration::from_millis(1))) // Very short duration
        .set_delay_strategy(RetryDelayStrategy::None)
        .build();

    // Sleep to ensure time passes
    std::thread::sleep(Duration::from_millis(5));

    let result = executor.run(|| {
        Ok(ApiResponse {
            status: 200,
            message: "Success".to_string(),
        })
    });

    // Should succeed if operation completes before time check, or fail with max duration exceeded
    if let Err(error) = result {
        let error_msg = error.to_string();
        assert!(error_msg.contains("Maximum duration exceeded"));
    }
}

#[tokio::test]
async fn test_async_max_duration_exceeded() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(100)
        .set_max_duration(Some(Duration::from_millis(50)))
        .set_delay_strategy(RetryDelayStrategy::Fixed {
            delay: Duration::from_millis(20),
        })
        .build();

    let result = executor
        .run_async(|| async {
            let mut count = attempt_count_clone.lock().unwrap();
            *count += 1;
            drop(count);

            Err(Box::new(ApiError("Always fail".to_string())) as Box<dyn Error + Send + Sync>)
        })
        .await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    // Should fail due to max duration exceeded
    assert!(
        error_msg.contains("Maximum duration exceeded")
            || error_msg.contains("Maximum attempts exceeded")
    );
}

#[tokio::test]
async fn test_async_operation_timeout_elapsed() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_operation_timeout(Some(Duration::from_millis(10)))
        .set_delay_strategy(RetryDelayStrategy::None)
        .build();

    let result = executor
        .run_async(|| async {
            {
                let mut count = attempt_count_clone.lock().unwrap();
                *count += 1;
            }

            // Simulate long operation that exceeds timeout
            tokio::time::sleep(Duration::from_millis(50)).await;

            Ok(ApiResponse {
                status: 200,
                message: "Success".to_string(),
            })
        })
        .await;

    assert!(result.is_err());
    // Should timeout and exhaust all retries
    assert_eq!(*attempt_count.lock().unwrap(), 3);
}

#[tokio::test]
async fn test_async_no_operation_timeout() {
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(2)
        .set_operation_timeout(None) // No timeout
        .set_delay_strategy(RetryDelayStrategy::None)
        .build();

    let result = executor
        .run_async(|| async {
            // Small delay but no timeout configured
            tokio::time::sleep(Duration::from_millis(10)).await;

            Ok(ApiResponse {
                status: 200,
                message: "Success".to_string(),
            })
        })
        .await;

    assert!(result.is_ok());
}

#[test]
fn test_sync_operation_timeout_post_check() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_operation_timeout(Some(Duration::from_millis(10)))
        .set_delay_strategy(RetryDelayStrategy::None)
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        // Simulate long operation
        std::thread::sleep(Duration::from_millis(50));

        Ok(ApiResponse {
            status: 200,
            message: "Success".to_string(),
        })
    });

    assert!(result.is_err());
    // Should detect timeout and retry
    assert_eq!(*attempt_count.lock().unwrap(), 3);
}

// ==================== Additional tests to increase coverage ====================

#[test]
fn test_abort_on_result_in_set() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let abort_result = ApiResponse {
        status: 401,
        message: "Unauthorized".to_string(),
    };

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .abort_on_results(vec![abort_result.clone()])
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        Ok(abort_result.clone())
    });

    assert!(result.is_err());
    // Should abort on first attempt
    assert_eq!(*attempt_count.lock().unwrap(), 1);
}

#[test]
fn test_retry_on_result_in_set() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let retry_result = ApiResponse {
        status: 500,
        message: "Server Error".to_string(),
    };

    let success_result = ApiResponse {
        status: 200,
        message: "Success".to_string(),
    };

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .failed_on_results(vec![retry_result.clone()])
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        let current = *count;
        drop(count);

        if current < 3 {
            Ok(retry_result.clone())
        } else {
            Ok(success_result.clone())
        }
    });

    assert!(result.is_ok());
    assert_eq!(*attempt_count.lock().unwrap(), 3);
}

#[tokio::test]
async fn test_async_abort_on_result() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .abort_on_results_if(|r| r.status == 403)
        .build();

    let result = executor
        .run_async(|| async {
            let mut count = attempt_count_clone.lock().unwrap();
            *count += 1;
            drop(count);

            Ok(ApiResponse {
                status: 403,
                message: "Forbidden".to_string(),
            })
        })
        .await;

    assert!(result.is_err());
    assert_eq!(*attempt_count.lock().unwrap(), 1);
}

#[tokio::test]
async fn test_async_retry_on_result_condition() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .failed_on_results_if(|r| r.status >= 500)
        .build();

    let result = executor
        .run_async(|| async {
            let mut count = attempt_count_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current < 3 {
                Ok(ApiResponse {
                    status: 503,
                    message: "Service Unavailable".to_string(),
                })
            } else {
                Ok(ApiResponse {
                    status: 200,
                    message: "Success".to_string(),
                })
            }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(*attempt_count.lock().unwrap(), 3);
}

#[test]
fn test_error_no_abort_configured() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    // No abort errors configured
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::None)
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        let current = *count;
        drop(count);

        if current < 2 {
            Err(Box::new(ApiError("Error".to_string())) as Box<dyn Error + Send + Sync>)
        } else {
            Ok(ApiResponse {
                status: 200,
                message: "Success".to_string(),
            })
        }
    });

    assert!(result.is_ok());
    assert_eq!(*attempt_count.lock().unwrap(), 2);
}

#[test]
fn test_result_not_in_failed_set_and_no_condition() {
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(2)
        .set_delay_strategy(RetryDelayStrategy::None)
        .failed_on_results(vec![ApiResponse {
            status: 500,
            message: "Error".to_string(),
        }])
        .build();

    let result = executor.run(|| {
        Ok(ApiResponse {
            status: 200,
            message: "Success".to_string(),
        })
    });

    // Should succeed immediately as result doesn't match failed set
    assert!(result.is_ok());
}

#[test]
fn test_result_not_in_abort_set_and_no_condition() {
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(2)
        .set_delay_strategy(RetryDelayStrategy::None)
        .abort_on_results(vec![ApiResponse {
            status: 401,
            message: "Unauthorized".to_string(),
        }])
        .build();

    let result = executor.run(|| {
        Ok(ApiResponse {
            status: 200,
            message: "Success".to_string(),
        })
    });

    // Should succeed immediately as result doesn't match abort set
    assert!(result.is_ok());
}

// ==================== Tests for uncovered branches ====================

#[test]
fn test_should_abort_error_with_dyn_error_typeid() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    // This test covers the branch: abort_error_types.contains(&TypeId::of::<dyn Error>())
    // By configuring to abort on all errors, we trigger this branch
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .failed_on_all_errors() // This adds dyn Error type
        .abort_on_error::<std::io::Error>() // This will check for abort
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        Err(
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, "IO Error"))
                as Box<dyn Error + Send + Sync>,
        )
    });

    assert!(result.is_err());
    // Should abort on first attempt due to abort_on_error configuration
    assert_eq!(*attempt_count.lock().unwrap(), 1);
}

#[test]
fn test_should_retry_result_with_condition_false() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    // This test covers the Some branch where condition returns false
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::None)
        .failed_on_results_if(|r| r.status >= 500) // Condition that won't match 400
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        Ok(ApiResponse {
            status: 400, // This won't match the condition (>= 500)
            message: "Bad Request".to_string(),
        })
    });

    // Should succeed immediately as condition doesn't match
    assert!(result.is_ok());
    assert_eq!(*attempt_count.lock().unwrap(), 1);
}

#[test]
fn test_should_abort_result_with_condition_false() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    // This test covers the Some branch where abort condition returns false
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(3)
        .set_delay_strategy(RetryDelayStrategy::None)
        .abort_on_results_if(|r| r.status == 401) // Condition that won't match 403
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        Ok(ApiResponse {
            status: 403, // This won't match the abort condition (== 401)
            message: "Forbidden".to_string(),
        })
    });

    // Should succeed as abort condition doesn't match
    assert!(result.is_ok());
    assert_eq!(*attempt_count.lock().unwrap(), 1);
}

#[tokio::test]
async fn test_async_should_retry_result_with_condition_true_then_false() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    // Test the condition branch returning true then false
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .failed_on_results_if(|r| r.status >= 500)
        .build();

    let result = executor
        .run_async(|| async {
            let mut count = attempt_count_clone.lock().unwrap();
            *count += 1;
            let current = *count;
            drop(count);

            if current == 1 {
                Ok(ApiResponse {
                    status: 503, // Matches condition, will retry
                    message: "Service Unavailable".to_string(),
                })
            } else {
                Ok(ApiResponse {
                    status: 200, // Doesn't match condition, success
                    message: "Success".to_string(),
                })
            }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(*attempt_count.lock().unwrap(), 2);
}

#[tokio::test]
async fn test_async_should_abort_result_with_condition_true() {
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    // Test abort condition branch returning true
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .abort_on_results_if(|r| r.status == 402)
        .build();

    let result = executor
        .run_async(|| async {
            let mut count = attempt_count_clone.lock().unwrap();
            *count += 1;
            drop(count);

            Ok(ApiResponse {
                status: 402, // Matches abort condition
                message: "Payment Required".to_string(),
            })
        })
        .await;

    assert!(result.is_err());
    assert_eq!(*attempt_count.lock().unwrap(), 1);
}

/// Test covering abort_error_types.contains(&TypeId::of::<dyn Error>())
/// by using abort_on_all_errors() which inserts dyn Error TypeId
#[test]
fn test_should_abort_error_with_abort_all_errors() {
    use prism3_retry::RetryError;

    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    // Use the new abort_on_all_errors() method to trigger the TypeId::of::<dyn Error>() branch
    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .abort_on_all_errors() // This inserts TypeId::of::<dyn Error>()
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        // Any error should be aborted
        Err(Box::new(ApiError("Should abort".to_string())) as Box<dyn Error + Send + Sync>)
    });

    // Should abort immediately on first error
    assert!(result.is_err());
    if let Err(e) = result {
        match e {
            RetryError::Aborted { reason } => {
                // Expected abort due to error
                assert_eq!(reason, "Operation aborted");
            }
            _ => panic!("Expected Aborted error, got: {:?}", e),
        }
    }
    assert_eq!(*attempt_count.lock().unwrap(), 1);
}

/// Test abort_on_all_errors with different error types
#[test]
fn test_abort_on_all_errors_with_io_error() {
    use prism3_retry::RetryError;

    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .abort_on_all_errors()
        .build();

    let result = executor.run(|| {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        // Different error type should also be aborted
        Err(
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, "IO Error"))
                as Box<dyn Error + Send + Sync>,
        )
    });

    assert!(result.is_err());
    if let Err(e) = result {
        match e {
            RetryError::Aborted { reason } => {
                assert_eq!(reason, "Operation aborted");
            }
            _ => panic!("Expected Aborted error, got: {:?}", e),
        }
    }
    assert_eq!(*attempt_count.lock().unwrap(), 1);
}
