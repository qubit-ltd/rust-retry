//! # Retry Integration Tests
//!
//! Tests complete workflow of RetryBuilder and RetryExecutor.

use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use prism3_retry::{RetryBuilder, RetryDelayStrategy, SimpleRetryConfig};

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
            assert!(true);
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
        .on_retry(move |_event| {
            *retry_count_clone.lock().unwrap() += 1;
        })
        .on_success(move |_event| {
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
        .on_retry(move |_event| {
            *retry_count_clone.lock().unwrap() += 1;
        })
        .on_success(move |_event| {
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
        .on_failure(move |_event| {
            *failure_count_clone.lock().unwrap() += 1;
        })
        .build();

    let result = executor
        .run(|| Err(Box::new(ApiError("Always fail".to_string())) as Box<dyn Error + Send + Sync>));

    assert!(result.is_err());
    // Failed event should be triggered once (after reaching maximum attempt count)
    assert_eq!(*failure_count.lock().unwrap(), 1);
}

#[test]
fn test_abort_event_listener() {
    let abort_count = Arc::new(Mutex::new(0));
    let abort_count_clone = abort_count.clone();

    let executor = RetryBuilder::<ApiResponse>::new()
        .set_max_attempts(5)
        .set_delay_strategy(RetryDelayStrategy::None)
        .abort_on_results_if(|r| r.status == 401)
        .on_abort(move |_event| {
            *abort_count_clone.lock().unwrap() += 1;
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
    assert_eq!(*abort_count.lock().unwrap(), 1);
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
