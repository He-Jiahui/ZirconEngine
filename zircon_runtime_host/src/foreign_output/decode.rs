//! Bounded JSON decoding for foreign runtime payloads.

use std::io::{self, BufReader, Read};
use std::time::{Duration, Instant};

use serde::de::{DeserializeOwned, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

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
    let deadline = decode_started + budget.max_decode_time;
    // The interface item limit counts typed rows/deliveries, while the JSON graph also contains
    // envelopes and arbitrary payload values. Bound the allocation-free syntax pass by the wire
    // ceiling, then apply the exact typed item policy below.
    let json_value_limit = budget.max_encoded_bytes.saturating_add(1);
    if let Err(error) = preflight_json_graph(bytes, json_value_limit, deadline, operation) {
        return (Err(error), decode_started.elapsed());
    }
    let mut timed_out = false;
    let decoded = {
        let reader = DeadlineReader::new(bytes, deadline, &mut timed_out);
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
        budget.validate_decode_duration(decode_started.elapsed(), operation)?;
        budget.validate_item_count(item_count, operation)?;
        budget.validate_decode_duration(decode_started.elapsed(), operation)?;
        Ok(decoded)
    });
    let decode_time = decode_started.elapsed();
    let decoded = decoded.and_then(|decoded| {
        budget.validate_decode_duration(decode_time, operation)?;
        Ok(decoded)
    });
    (decoded, decode_time)
}

fn preflight_json_graph(
    bytes: &[u8],
    max_json_values: usize,
    deadline: Instant,
    operation: &'static str,
) -> Result<(), RuntimeForeignOutputError> {
    let mut timed_out = false;
    let mut counter = JsonItemCounter::new(max_json_values);
    let reader = DeadlineReader::new(bytes, deadline, &mut timed_out);
    let mut deserializer = serde_json::Deserializer::from_reader(BufReader::with_capacity(
        DECODE_READER_CHUNK_BYTES,
        reader,
    ));
    let result = JsonItemSeed {
        counter: &mut counter,
    }
    .deserialize(&mut deserializer)
    .and_then(|()| deserializer.end());
    if timed_out {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} exceeded its decode time budget during item preflight"
        )));
    }
    if let Some(observed) = counter.overflow_observed {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned {observed} JSON values; syntax-graph maximum is {max_json_values}"
        )));
    }
    result.map_err(|error| {
        RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} failed JSON item preflight (maximum nesting depth {RUNTIME_FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH}): {error}"
        ))
    })
}

struct JsonItemCounter {
    observed: usize,
    limit: usize,
    overflow_observed: Option<usize>,
}

impl JsonItemCounter {
    fn new(limit: usize) -> Self {
        Self {
            observed: 0,
            limit,
            overflow_observed: None,
        }
    }

    fn observe<E: serde::de::Error>(&mut self) -> Result<(), E> {
        self.observed = self.observed.saturating_add(1);
        if self.observed <= self.limit {
            return Ok(());
        }
        self.overflow_observed = Some(self.observed);
        Err(E::custom("JSON item limit exceeded"))
    }
}

struct JsonItemSeed<'a> {
    counter: &'a mut JsonItemCounter,
}

impl<'de> DeserializeSeed<'de> for JsonItemSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.counter.observe::<D::Error>()?;
        deserializer.deserialize_any(JsonItemVisitor {
            counter: self.counter,
        })
    }
}

struct JsonItemVisitor<'a> {
    counter: &'a mut JsonItemCounter,
}

impl<'de> Visitor<'de> for JsonItemVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_borrowed_str<E>(self, _value: &'de str) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_string<E>(self, _value: String) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        JsonItemSeed {
            counter: self.counter,
        }
        .deserialize(deserializer)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        JsonItemSeed {
            counter: self.counter,
        }
        .deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while sequence
            .next_element_seed(JsonItemSeed {
                counter: self.counter,
            })?
            .is_some()
        {}
        Ok(())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while map.next_key::<IgnoredAny>()?.is_some() {
            map.next_value_seed(JsonItemSeed {
                counter: self.counter,
            })?;
        }
        Ok(())
    }
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
