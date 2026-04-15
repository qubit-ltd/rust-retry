/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use qubit_function::{ArcBiFunction, BiFunction};
use qubit_retry::{RetryAttemptContext, RetryDecider, RetryDecision, RetryExecutor};

/// Ensures [`RetryDecider`] can be built and supplied to the executor builder.
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
///
/// # Errors
/// The test fails through assertions when wiring is incorrect.
#[test]
fn test_retry_decider_constant_retry_builds_executor() {
    let decider: RetryDecider<std::io::Error> = ArcBiFunction::constant(RetryDecision::Retry);
    let _ = decider.apply(
        &std::io::Error::other("x"),
        &RetryAttemptContext {
            attempt: 1,
            max_attempts: 3,
            elapsed: std::time::Duration::ZERO,
        },
    );

    let executor = RetryExecutor::<std::io::Error>::builder()
        .retry_decide(decider)
        .build()
        .expect("executor should build");
    let _ = executor;
}
