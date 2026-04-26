/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
#![cfg(feature = "tokio")]

use std::time::Duration;

use qubit_retry::{AttemptFailure, Retry, RetryErrorReason};

use crate::support::TestError;

/// Verifies async operation panic still propagates through the current task.
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
#[tokio::test]
#[should_panic(expected = "async operation panic")]
async fn test_run_async_panic_propagates() {
    let retry = Retry::<TestError>::builder()
        .max_attempts(2)
        .no_delay()
        .build()
        .expect("retry should build");

    let _ = retry
        .run_async::<(), _, _>(|| async { panic!("async operation panic") })
        .await;
}

/// Verifies async attempt timeout becomes a retry failure.
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
#[tokio::test]
async fn test_run_async_attempt_timeout_can_abort() {
    let retry = Retry::<TestError>::builder()
        .max_attempts(3)
        .attempt_timeout(Some(Duration::from_millis(1)))
        .abort_on_timeout()
        .no_delay()
        .build()
        .expect("retry should build");

    let error = retry
        .run_async(|| async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            Ok::<(), TestError>(())
        })
        .await
        .expect_err("timeout should abort");

    assert_eq!(error.reason(), RetryErrorReason::Aborted);
    assert!(matches!(
        error.last_failure(),
        Some(AttemptFailure::Timeout)
    ));
    assert_eq!(
        error.context().attempt_timeout(),
        Some(Duration::from_millis(1))
    );
}

/// Verifies async retry succeeds without per-attempt timeout after a retry delay.
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
///
/// # Errors
/// The test fails through assertions when async retry does not reach success.
#[tokio::test(start_paused = true)]
async fn test_run_async_without_timeout_retries_until_success() {
    let retry = Retry::<TestError>::builder()
        .max_attempts(2)
        .fixed_delay(Duration::from_millis(1))
        .build()
        .expect("retry should build");
    let mut attempts = 0;

    let value = retry
        .run_async(|| {
            attempts += 1;
            let current_attempt = attempts;
            async move {
                if current_attempt == 1 {
                    Err(TestError("temporary"))
                } else {
                    Ok("done")
                }
            }
        })
        .await
        .expect("second async attempt should succeed");

    assert_eq!(value, "done");
    assert_eq!(attempts, 2);
}

/// Verifies async timeout wrapping preserves fast successful results.
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
///
/// # Errors
/// The test fails through assertions when timeout wrapping changes success output.
#[tokio::test(start_paused = true)]
async fn test_run_async_with_attempt_timeout_allows_fast_success() {
    let retry = Retry::<TestError>::builder()
        .max_attempts(1)
        .attempt_timeout(Some(Duration::from_millis(10)))
        .no_delay()
        .build()
        .expect("retry should build");

    let value = retry
        .run_async(|| async { Ok::<_, TestError>("fast") })
        .await
        .expect("fast async attempt should succeed");

    assert_eq!(value, "fast");
}

/// Verifies async execution can stop before the first attempt on elapsed budget.
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
///
/// # Errors
/// The test fails through assertions when async elapsed-budget handling differs.
#[tokio::test]
async fn test_run_async_max_elapsed_can_stop_before_first_attempt() {
    let retry = Retry::<TestError>::builder()
        .max_elapsed(Some(Duration::ZERO))
        .attempt_timeout(Some(Duration::from_millis(1)))
        .no_delay()
        .build()
        .expect("retry should build");

    let error = retry
        .run_async::<(), _, _>(|| async { panic!("operation must not run") })
        .await
        .expect_err("zero elapsed budget should stop before first attempt");

    assert_eq!(error.reason(), RetryErrorReason::MaxElapsedExceeded);
    assert_eq!(error.attempts(), 0);
    assert_eq!(
        error.context().attempt_timeout(),
        Some(Duration::from_millis(1))
    );
}

/// Verifies async retry handles zero retry delay without sleeping.
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
///
/// # Errors
/// The test fails through assertions when zero-delay async retry does not proceed.
#[tokio::test]
async fn test_run_async_zero_delay_retry_skips_sleep() {
    let retry = Retry::<TestError>::builder()
        .max_attempts(2)
        .no_delay()
        .build()
        .expect("retry should build");
    let mut attempts = 0;

    let value = retry
        .run_async(|| {
            attempts += 1;
            let current_attempt = attempts;
            async move {
                if current_attempt == 1 {
                    Err(TestError("temporary"))
                } else {
                    Ok("done")
                }
            }
        })
        .await
        .expect("second async attempt should succeed");

    assert_eq!(value, "done");
    assert_eq!(attempts, 2);
}
