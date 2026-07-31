use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde_json::{Number, Value};

pub const FNV_OFFSET: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;
const FLOAT_QUANTUM: f64 = 1_000_000.0;

#[derive(Clone, Debug, PartialEq)]
pub enum TraceValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<TraceValue>),
    Object(BTreeMap<String, TraceValue>),
    Map(Vec<(TraceValue, TraceValue)>),
    Set(Vec<TraceValue>),
}

pub fn round6(value: f64) -> Value {
    if value.is_nan() {
        return Value::String("NaN".to_string());
    }
    if value == f64::INFINITY {
        return Value::String("Infinity".to_string());
    }
    if value == f64::NEG_INFINITY {
        return Value::String("-Infinity".to_string());
    }
    if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
        return Value::Number(Number::from(value as i64));
    }
    let rounded = ((value * FLOAT_QUANTUM) + 0.5).floor() / FLOAT_QUANTUM;
    let normalized = if rounded == 0.0 { 0.0 } else { rounded };
    if normalized.fract() == 0.0 && normalized >= i64::MIN as f64 && normalized <= i64::MAX as f64 {
        return Value::Number(Number::from(normalized as i64));
    }
    Value::Number(Number::from_f64(normalized).expect("rounded finite value must remain finite"))
}

pub fn canonical(value: &TraceValue, omit_defaults: bool) -> Value {
    match value {
        TraceValue::Null => Value::Null,
        TraceValue::Bool(value) => Value::Bool(*value),
        TraceValue::Number(value) => round6(*value),
        TraceValue::String(value) => Value::String(value.clone()),
        TraceValue::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| canonical(value, omit_defaults))
                .collect(),
        ),
        TraceValue::Object(values) => {
            let mut object = serde_json::Map::new();
            for (key, value) in values {
                let value = canonical(value, omit_defaults);
                if omit_defaults && is_inert(&value) {
                    continue;
                }
                object.insert(key.clone(), value);
            }
            Value::Object(object)
        }
        TraceValue::Map(values) => {
            let mut entries = values
                .iter()
                .map(|(key, value)| {
                    (
                        canonical(key, omit_defaults),
                        canonical(value, omit_defaults),
                    )
                })
                .collect::<Vec<_>>();
            entries.sort_by(|left, right| compare_keys(&left.0, &right.0));
            Value::Array(
                entries
                    .into_iter()
                    .map(|(key, value)| Value::Array(vec![key, value]))
                    .collect(),
            )
        }
        TraceValue::Set(values) => {
            let mut values = values
                .iter()
                .map(|value| canonical(value, omit_defaults))
                .collect::<Vec<_>>();
            values.sort_by(compare_keys);
            Value::Array(values)
        }
    }
}

pub fn fnv1a_hex(value: &str) -> String {
    format!("{:08x}", fnv1a_utf16(value))
}

pub fn fnv1a_step_u32(mut hash: u32, value: u32) -> u32 {
    for byte in value.to_le_bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

pub fn digest(value: &TraceValue, omit_defaults: bool) -> String {
    let canonical = canonical(value, omit_defaults);
    fnv1a_hex(&serde_json::to_string(&canonical).expect("canonical trace must serialize"))
}

fn fnv1a_utf16(value: &str) -> u32 {
    let mut hash = FNV_OFFSET;
    for code_unit in value.encode_utf16() {
        hash ^= u32::from(code_unit);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn compare_keys(left: &Value, right: &Value) -> Ordering {
    match (left.as_f64(), right.as_f64()) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        _ => serde_json::to_string(left)
            .expect("canonical key must serialize")
            .cmp(&serde_json::to_string(right).expect("canonical key must serialize")),
    }
}

fn is_inert(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(false) => true,
        Value::Number(number) => number.as_f64() == Some(0.0),
        Value::String(value) => value.is_empty(),
        Value::Array(values) => values.is_empty(),
        Value::Bool(true) | Value::Object(_) => false,
    }
}
