//! Bounded JSON decoding for foreign runtime payloads.

use std::io::{self, BufReader, Read};
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;

use super::{
    RuntimeForeignOutputBudget, RuntimeForeignOutputError,
    RUNTIME_FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH,
};

const DECODE_READER_CHUNK_BYTES: usize = 4 * 1024;

pub(super) fn decode_bounded_json<T, E>(
    bytes: &[u8],
    budget: RuntimeForeignOutputBudget,
    operation: &'static str,
    validate: impl FnOnce(&T) -> Result<usize, E>,
) -> (Result<T, RuntimeForeignOutputError>, Duration)
where
    T: DeserializeOwned,
    E: std::fmt::Display,
{
    let decode_started = Instant::now();
    let mut timed_out = false;
    let decoded = {
        let reader = DeadlineReader::new(
            bytes,
            decode_started + budget.max_decode_time,
            &mut timed_out,
        );
        serde_json::from_reader::<_, T>(BufReader::with_capacity(DECODE_READER_CHUNK_BYTES, reader))
    };
    let decoded = if timed_out {
        Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} exceeded its decode time budget while parsing: maximum is {} microseconds",
            budget.max_decode_time.as_micros()
        )))
    } else {
        decoded.map_err(|error| {
            RuntimeForeignOutputError::protocol_violation(format!(
                "{operation} failed bounded JSON decode (maximum nesting depth {RUNTIME_FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH}): {error}"
            ))
        })
    };
    let decoded = decoded.and_then(|decoded| {
        let item_count = validate(&decoded)
            .map_err(|error| RuntimeForeignOutputError::protocol_violation(error.to_string()))?;
        budget.validate_item_count(item_count, operation)?;
        Ok(decoded)
    });
    let decode_time = decode_started.elapsed();
    let decoded = decoded.and_then(|decoded| {
        budget.validate_decode_duration(decode_time, operation)?;
        Ok(decoded)
    });
    (decoded, decode_time)
}

struct DeadlineReader<'a> {
    remaining: &'a [u8],
    deadline: Instant,
    timed_out: &'a mut bool,
}

impl<'a> DeadlineReader<'a> {
    fn new(remaining: &'a [u8], deadline: Instant, timed_out: &'a mut bool) -> Self {
        Self {
            remaining,
            deadline,
            timed_out,
        }
    }
}

impl Read for DeadlineReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.remaining.is_empty() || output.is_empty() {
            return Ok(0);
        }
        if Instant::now() >= self.deadline {
            *self.timed_out = true;
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "foreign output decode deadline exceeded",
            ));
        }
        let count = output
            .len()
            .min(self.remaining.len())
            .min(DECODE_READER_CHUNK_BYTES);
        output[..count].copy_from_slice(&self.remaining[..count]);
        self.remaining = &self.remaining[count..];
        Ok(count)
    }
}
