#![warn(unused)]
use chrono::{DateTime, Utc};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::error::{FetchError::ResponseError, FetchResult};


pub fn unix_timestamp_to_instant(value: u64) -> Instant {
    let target = UNIX_EPOCH + Duration::from_millis(value as u64);
    let remaining = target.duration_since(SystemTime::now())
        .unwrap_or(Duration::ZERO);

    Instant::now() + remaining
}

pub fn iso_8601_to_instant(value: &str) -> FetchResult<Instant> {
    let datetime = value.parse::<DateTime<Utc>>()
        .map_err(|e| ResponseError(format!("Failed to convert header value: {e}")))?;

    let target: SystemTime = datetime.into();
    let remaining = target.duration_since(SystemTime::now())
        .unwrap_or(Duration::ZERO);

    let instant = Instant::now() + remaining;

    Ok(instant)
}