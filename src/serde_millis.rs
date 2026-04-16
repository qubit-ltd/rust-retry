/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Serde helpers: [`std::time::Duration`] as whole milliseconds (`u64`).

use serde::{Deserialize, Deserializer, Serializer};
use std::time::Duration;

/// Serializes a [`Duration`] as milliseconds (`u64`).
pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let millis = duration.as_millis().min(u128::from(u64::MAX)) as u64;
    serializer.serialize_u64(millis)
}

/// Deserializes a [`Duration`] from milliseconds (`u64`).
pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let millis = u64::deserialize(deserializer)?;
    Ok(Duration::from_millis(millis))
}
