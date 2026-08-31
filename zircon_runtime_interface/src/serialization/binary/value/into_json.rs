use serde_json::{map::Entry, Map, Number, Value};

use super::super::wire::{
    MAX_BINARY_CONTAINER_ENTRIES, MAX_BINARY_DEPTH, MAX_BINARY_NODES, MAX_BINARY_STRING_BYTES,
};
use super::{BinaryNode, BinaryValue, BinaryValueError};

enum DecodeFrame {
    Array {
        remaining: usize,
        values: Vec<Value>,
    },
    Object {
        remaining: usize,
        values: Map<String, Value>,
        pending_key: Option<String>,
    },
}

impl TryFrom<BinaryValue> for Value {
    type Error = BinaryValueError;

    fn try_from(value: BinaryValue) -> Result<Self, Self::Error> {
        if value.nodes.len() > MAX_BINARY_NODES {
            return Err(BinaryValueError::NodeLimitExceeded {
                max: MAX_BINARY_NODES,
                found: value.nodes.len(),
            });
        }

        let mut frames = Vec::new();
        let mut root = None;
        for node in value.nodes {
            match node {
                BinaryNode::ObjectKey(key) => assign_object_key(&mut frames, key)?,
                BinaryNode::Array { len } => {
                    let len = validate_container(len)?;
                    validate_depth(frames.len() + 1)?;
                    if len == 0 {
                        attach_value(Value::Array(Vec::new()), &mut frames, &mut root)?;
                    } else {
                        frames.push(DecodeFrame::Array {
                            remaining: len,
                            values: Vec::with_capacity(len.min(1024)),
                        });
                    }
                }
                BinaryNode::Object { len } => {
                    let len = validate_container(len)?;
                    validate_depth(frames.len() + 1)?;
                    if len == 0 {
                        attach_value(Value::Object(Map::new()), &mut frames, &mut root)?;
                    } else {
                        frames.push(DecodeFrame::Object {
                            remaining: len,
                            values: Map::new(),
                            pending_key: None,
                        });
                    }
                }
                node => attach_value(primitive_value(node)?, &mut frames, &mut root)?,
            }
        }

        if !frames.is_empty() {
            return Err(BinaryValueError::IncompleteContainer);
        }
        root.ok_or(BinaryValueError::EmptyValue)
    }
}

fn primitive_value(node: BinaryNode) -> Result<Value, BinaryValueError> {
    match node {
        BinaryNode::Null => Ok(Value::Null),
        BinaryNode::Bool(value) => Ok(Value::Bool(value)),
        BinaryNode::I64(value) => Ok(Value::Number(Number::from(value))),
        BinaryNode::U64(value) => Ok(Value::Number(Number::from(value))),
        BinaryNode::F64(value) => Number::from_f64(value)
            .map(Value::Number)
            .ok_or(BinaryValueError::NonFiniteFloat { value }),
        BinaryNode::String(value) => {
            validate_string(&value)?;
            Ok(Value::String(value))
        }
        BinaryNode::Array { .. } => Err(BinaryValueError::UnexpectedNodeKind { kind: "array" }),
        BinaryNode::Object { .. } => Err(BinaryValueError::UnexpectedNodeKind { kind: "object" }),
        BinaryNode::ObjectKey(_) => {
            Err(BinaryValueError::UnexpectedNodeKind { kind: "object-key" })
        }
    }
}

fn assign_object_key(frames: &mut [DecodeFrame], key: String) -> Result<(), BinaryValueError> {
    validate_string(&key)?;
    let Some(DecodeFrame::Object { pending_key, .. }) = frames.last_mut() else {
        return Err(BinaryValueError::UnexpectedObjectKey { key });
    };
    if pending_key.is_some() {
        return Err(BinaryValueError::UnexpectedObjectKey { key });
    }
    *pending_key = Some(key);
    Ok(())
}

fn attach_value(
    mut value: Value,
    frames: &mut Vec<DecodeFrame>,
    root: &mut Option<Value>,
) -> Result<(), BinaryValueError> {
    loop {
        let Some(frame) = frames.last_mut() else {
            if root.replace(value).is_some() {
                return Err(BinaryValueError::MultipleRootValues);
            }
            return Ok(());
        };
        let completed = match frame {
            DecodeFrame::Array { remaining, values } => {
                values.push(value);
                *remaining -= 1;
                (*remaining == 0).then(|| Value::Array(std::mem::take(values)))
            }
            DecodeFrame::Object {
                remaining,
                values,
                pending_key,
            } => {
                let key = pending_key
                    .take()
                    .ok_or(BinaryValueError::MissingObjectKey)?;
                insert_unique_object_value(values, key, value)?;
                *remaining -= 1;
                (*remaining == 0).then(|| Value::Object(std::mem::take(values)))
            }
        };
        let Some(container) = completed else {
            return Ok(());
        };
        frames.pop();
        value = container;
    }
}

fn insert_unique_object_value(
    values: &mut Map<String, Value>,
    key: String,
    value: Value,
) -> Result<(), BinaryValueError> {
    match values.entry(key) {
        Entry::Vacant(entry) => {
            entry.insert(value);
            Ok(())
        }
        Entry::Occupied(entry) => Err(BinaryValueError::DuplicateObjectKey {
            key: entry.key().clone(),
        }),
    }
}

fn validate_container(found: u32) -> Result<usize, BinaryValueError> {
    let found = found as usize;
    if found <= MAX_BINARY_CONTAINER_ENTRIES {
        return Ok(found);
    }
    Err(BinaryValueError::ContainerLimitExceeded {
        max: MAX_BINARY_CONTAINER_ENTRIES,
        found,
    })
}

fn validate_depth(found: usize) -> Result<(), BinaryValueError> {
    if found <= MAX_BINARY_DEPTH {
        return Ok(());
    }
    Err(BinaryValueError::DepthLimitExceeded {
        max: MAX_BINARY_DEPTH,
        found,
    })
}

fn validate_string(value: &str) -> Result<(), BinaryValueError> {
    if value.len() <= MAX_BINARY_STRING_BYTES {
        return Ok(());
    }
    Err(BinaryValueError::StringLimitExceeded {
        max: MAX_BINARY_STRING_BYTES,
        found: value.len(),
    })
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use serde_json::{Map, Value};

    use super::insert_unique_object_value;

    const PERF_ENTRY_COUNT: usize = 10_000;
    const PERF_SAMPLE_PAIRS: usize = 21;

    #[test]
    #[ignore = "release performance evidence"]
    fn binary_object_decode_uses_one_index_lookup_per_entry() {
        let keys = (0..PERF_ENTRY_COUNT)
            .map(|index| format!("field-{index:05}"))
            .collect::<Vec<_>>();
        let mut legacy_ns = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        let mut optimized_ns = Vec::with_capacity(PERF_SAMPLE_PAIRS);

        for sample in 0..PERF_SAMPLE_PAIRS {
            let legacy_keys = keys.clone();
            let optimized_keys = keys.clone();
            let (legacy, optimized) = if sample % 2 == 0 {
                (
                    measure_once(|| build_legacy_object(legacy_keys)),
                    measure_once(|| build_optimized_object(optimized_keys)),
                )
            } else {
                let optimized = measure_once(|| build_optimized_object(optimized_keys));
                let legacy = measure_once(|| build_legacy_object(legacy_keys));
                (legacy, optimized)
            };
            legacy_ns.push(legacy);
            optimized_ns.push(optimized);
        }

        let legacy_p50 = percentile(&legacy_ns, 50);
        let legacy_p95 = percentile(&legacy_ns, 95);
        let optimized_p50 = percentile(&optimized_ns, 50);
        let optimized_p95 = percentile(&optimized_ns, 95);
        println!(
            "PERF_RESULT runtime_interface02_binary_object_entry legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} object_entries={PERF_ENTRY_COUNT} samples={PERF_SAMPLE_PAIRS} legacy_index_lookups_per_entry=2 optimized_index_lookups_per_entry=1"
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(80),
            "optimized P95 {optimized_p95}ns must be at most 80% of legacy P95 {legacy_p95}ns"
        );
    }

    fn build_legacy_object(keys: Vec<String>) -> Map<String, Value> {
        let mut values = Map::new();
        for key in keys {
            assert!(!values.contains_key(&key));
            values.insert(key, Value::Null);
        }
        values
    }

    fn build_optimized_object(keys: Vec<String>) -> Map<String, Value> {
        let mut values = Map::new();
        for key in keys {
            insert_unique_object_value(&mut values, key, Value::Null)
                .expect("performance fixture keys are unique");
        }
        values
    }

    fn measure_once<T>(build: impl FnOnce() -> T) -> u64 {
        let started = Instant::now();
        black_box(build());
        started.elapsed().as_nanos() as u64
    }

    fn percentile(samples: &[u64], percentile: usize) -> u64 {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let rank = ordered
            .len()
            .saturating_mul(percentile)
            .div_ceil(100)
            .saturating_sub(1);
        ordered[rank]
    }
}
