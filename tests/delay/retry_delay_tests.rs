/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use std::convert::TryFrom;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::str::FromStr;
use std::time::Duration;

use qubit_retry::RetryDelay;
use serde_json;

/// Verifies every delay variant calculates the expected base delay.
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
///
/// # Errors
/// The test fails through assertions when a delay calculation is incorrect.
#[test]
fn test_base_delay_none_fixed_random_and_exponential_values() {
    assert_eq!(RetryDelay::none().base_delay(1), Duration::ZERO);
    assert_eq!(
        RetryDelay::fixed(Duration::from_millis(12)).base_delay(9),
        Duration::from_millis(12)
    );
    assert_eq!(
        RetryDelay::random(Duration::from_millis(7), Duration::from_millis(7)).base_delay(1),
        Duration::from_millis(7)
    );

    let random = RetryDelay::random(Duration::from_millis(5), Duration::from_millis(8));
    for _ in 0..20 {
        let delay = random.base_delay(1);
        assert!(delay >= Duration::from_millis(5));
        assert!(delay <= Duration::from_millis(8));
    }

    let exponential =
        RetryDelay::exponential(Duration::from_millis(100), Duration::from_millis(500), 2.0);
    assert_eq!(exponential.base_delay(0), Duration::from_millis(100));
    assert_eq!(exponential.base_delay(1), Duration::from_millis(100));
    assert_eq!(exponential.base_delay(2), Duration::from_millis(200));
    assert_eq!(exponential.base_delay(4), Duration::from_millis(500));
    assert_eq!(exponential.base_delay(u32::MAX), Duration::from_millis(500));
}

/// Verifies delay validation rejects invalid strategy parameters.
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
///
/// # Errors
/// The test fails through assertions when invalid values are accepted or valid
/// values are rejected.
#[test]
fn test_validate_rejects_invalid_values() {
    assert!(RetryDelay::fixed(Duration::ZERO).validate().is_err());
    assert!(RetryDelay::random(Duration::ZERO, Duration::from_millis(1))
        .validate()
        .is_err());
    assert!(
        RetryDelay::random(Duration::from_millis(2), Duration::from_millis(1))
            .validate()
            .is_err()
    );
    assert!(
        RetryDelay::random(Duration::from_millis(2), Duration::from_millis(2))
            .validate()
            .is_ok()
    );
    assert!(
        RetryDelay::exponential(Duration::ZERO, Duration::from_secs(1), 2.0)
            .validate()
            .is_err()
    );
    assert!(
        RetryDelay::exponential(Duration::from_secs(2), Duration::from_secs(1), 2.0)
            .validate()
            .is_err()
    );
    assert!(
        RetryDelay::exponential(Duration::from_secs(1), Duration::from_secs(2), 1.0)
            .validate()
            .is_err()
    );
    assert!(RetryDelay::exponential(
        Duration::from_secs(1),
        Duration::from_secs(2),
        f64::INFINITY
    )
    .validate()
    .is_err());
    assert!(RetryDelay::default().validate().is_ok());
}

/// Verifies [`std::fmt::Display`] for [`RetryDelay::None`] (`strum` snake_case).
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
///
/// # Errors
/// The test fails through assertions when the display string is incorrect.
#[test]
fn test_retry_delay_display_none() {
    assert_eq!(RetryDelay::none().to_string(), "none");
}

/// Documents current `strum` behavior: variants marked `#[strum(disabled)]` are
/// omitted from the generated [`std::fmt::Display`] match, so formatting
/// `Fixed` / `Random` / `Exponential` panics with a fixed message.
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
///
/// # Errors
/// The test fails through assertions when formatting does not panic as expected.
#[test]
fn test_retry_delay_display_strum_disabled_variants_panic_on_fmt() {
    let cases = [
        (
            "fixed",
            RetryDelay::fixed(Duration::from_millis(1)),
        ),
        (
            "random",
            RetryDelay::random(Duration::from_millis(1), Duration::from_millis(2)),
        ),
        (
            "exponential",
            RetryDelay::exponential(Duration::from_millis(10), Duration::from_millis(20), 2.0),
        ),
    ];
    for (name, delay) in cases {
        let err = catch_unwind(AssertUnwindSafe(|| {
            let _ = format!("{delay}");
        }))
        .unwrap_err();
        let msg = err
            .downcast_ref::<&'static str>()
            .copied()
            .or_else(|| err.downcast_ref::<String>().map(String::as_str))
            .unwrap_or("");
        assert!(
            msg.contains("disabled variant"),
            "unexpected panic payload for {name}: {msg:?}"
        );
    }
}

/// Verifies [`std::str::FromStr`] / [`std::convert::TryFrom<&str>`] for [`RetryDelay`]:
/// only the unit variant is registered with `strum::EnumString`; tuple variants are
/// `#[strum(disabled)]` and do not parse from plain text.
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
///
/// # Errors
/// The test fails through assertions when parsing does not match the expected rules.
#[test]
fn test_retry_delay_from_str_and_try_from_none_only() {
    assert_eq!(RetryDelay::from_str("none").unwrap(), RetryDelay::none());
    assert_eq!(
        RetryDelay::try_from("none").unwrap(),
        RetryDelay::none()
    );
    assert_eq!("none".parse::<RetryDelay>().unwrap(), RetryDelay::none());
}

/// Verifies [`std::str::FromStr`] rejects whitespace padding, unknown tokens, and
/// strings that resemble parameterized variants (those variants are not parseable).
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
///
/// # Errors
/// The test fails through assertions when invalid input is accepted.
#[test]
fn test_retry_delay_from_str_rejects_invalid_inputs() {
    for s in [
        "",
        " ",
        "  none  ",
        "NONE",
        "None",
        "nope",
        "fixed",
        "fixed(12ms)",
        "random(5ms..=8ms)",
        "exponential(initial=100ms, max=500ms, multiplier=2)",
    ] {
        assert!(
            RetryDelay::from_str(s).is_err(),
            "expected from_str error for {s:?}"
        );
    }
}

/// Verifies display → parse round-trip for [`RetryDelay::None`] only (other variants
/// are not `FromStr`-parseable by design).
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
///
/// # Errors
/// The test fails through assertions when the round-trip does not hold.
#[test]
fn test_retry_delay_display_parse_round_trip_none() {
    let delay = RetryDelay::none();
    let s = delay.to_string();
    assert_eq!(RetryDelay::from_str(&s).unwrap(), delay);
}

/// Verifies JSON serialization and deserialization for [`RetryDelay`] (millisecond
/// fields and `f64` multiplier) round-trip for representative values.
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
///
/// # Errors
/// The test fails through assertions or serde errors when JSON does not round-trip.
#[test]
fn test_retry_delay_serde_json_roundtrip_variants() {
    let cases = [
        RetryDelay::none(),
        RetryDelay::fixed(Duration::from_millis(12)),
        RetryDelay::random(Duration::from_millis(5), Duration::from_millis(8)),
        RetryDelay::exponential(Duration::from_millis(100), Duration::from_millis(500), 2.0),
        RetryDelay::default(),
    ];
    for original in cases {
        let json = serde_json::to_string(&original).unwrap();
        let parsed: RetryDelay = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, original, "json was {json}");
    }
}

/// Documents stable JSON shapes for [`RetryDelay`] literals used in configuration.
///
/// # Parameters
/// This test has no parameters.
///
/// # Returns
/// This test returns nothing.
///
/// # Errors
/// The test fails through assertions when serialized JSON drifts from expectations.
#[test]
fn test_retry_delay_serde_json_literal_shapes() {
    assert_eq!(serde_json::to_string(&RetryDelay::none()).unwrap(), r#""None""#);
    assert_eq!(
        serde_json::to_string(&RetryDelay::fixed(Duration::from_millis(12))).unwrap(),
        r#"{"Fixed":12}"#
    );
    assert_eq!(
        serde_json::to_string(&RetryDelay::random(
            Duration::from_millis(5),
            Duration::from_millis(8)
        ))
        .unwrap(),
        r#"{"Random":{"min":5,"max":8}}"#
    );
    assert_eq!(
        serde_json::to_string(&RetryDelay::exponential(
            Duration::from_millis(100),
            Duration::from_millis(500),
            2.0
        ))
        .unwrap(),
        r#"{"Exponential":{"initial":100,"max":500,"multiplier":2.0}}"#
    );
}
