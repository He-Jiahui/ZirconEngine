use std::collections::BTreeMap;

use serde_json::{json, Map, Value};
use thiserror::Error;

use crate::{canonical, fnv1a_hex, resolve_trace_symbol, TraceValue, TRACE_SYMBOL_FINGERPRINT};

const TRACE_MAGIC: [u8; 4] = *b"WTR1";
const TRACE_VERSION: u16 = 1;
const JS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

const TAG_NULL: u8 = 0;
const TAG_FALSE: u8 = 1;
const TAG_TRUE: u8 = 2;
const TAG_UNSIGNED: u8 = 3;
const TAG_SIGNED_MAGNITUDE: u8 = 4;
const TAG_FIXED6: u8 = 5;
const TAG_POSITIVE_INFINITY: u8 = 6;
const TAG_NEGATIVE_INFINITY: u8 = 7;
const TAG_NAN: u8 = 8;
const TAG_SYMBOL: u8 = 9;
const TAG_ARRAY: u8 = 10;
const TAG_OBJECT: u8 = 11;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VmTraceDecodeLimits {
    pub max_total_bytes: usize,
    pub max_value_depth: usize,
    pub max_collection_entries: usize,
    pub max_frames: usize,
    pub max_coverage_entries: usize,
}

impl Default for VmTraceDecodeLimits {
    fn default() -> Self {
        Self {
            max_total_bytes: 16 * 1024 * 1024,
            max_value_depth: 64,
            max_collection_entries: 1_000_000,
            max_frames: 4_096,
            max_coverage_entries: 4_096,
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum VmTraceWireError {
    #[error("VM trace exceeds {context} limit: actual {actual}, maximum {maximum}")]
    LimitExceeded {
        context: &'static str,
        actual: usize,
        maximum: usize,
    },
    #[error("VM trace header is truncated: actual {actual}, minimum {minimum}")]
    TruncatedHeader { actual: usize, minimum: usize },
    #[error("VM trace magic is invalid: {actual:?}")]
    InvalidMagic { actual: [u8; 4] },
    #[error("VM trace version {actual} is unsupported; expected {expected}")]
    UnsupportedVersion { actual: u16, expected: u16 },
    #[error(
        "VM trace dictionary fingerprint mismatch: actual {actual:015x}, expected {expected:015x}"
    )]
    DictionaryMismatch { actual: u64, expected: u64 },
    #[error("VM trace payload is truncated at {context}: needed {needed}, remaining {remaining}")]
    TruncatedPayload {
        context: &'static str,
        needed: usize,
        remaining: usize,
    },
    #[error("VM trace boolean has invalid byte {0}")]
    InvalidBoolean(u8),
    #[error("VM trace value has unknown tag {0}")]
    UnknownValueTag(u8),
    #[error("VM trace references unknown symbol {0}")]
    UnknownSymbol(u16),
    #[error("VM trace object repeats key symbol {symbol}")]
    DuplicateObjectKey { symbol: u16 },
    #[error("VM trace {context} must be an array")]
    ExpectedArray { context: &'static str },
    #[error("VM trace {context} must be a number")]
    ExpectedNumber { context: &'static str },
    #[error("VM trace number at {context} exceeds JavaScript safe integer range: {magnitude}")]
    NumberOutOfRange {
        context: &'static str,
        magnitude: u64,
    },
    #[error("VM trace contains {remaining} trailing bytes")]
    TrailingBytes { remaining: usize },
}

pub fn decode_vm_trace(
    bytes: &[u8],
    limits: VmTraceDecodeLimits,
) -> Result<Value, VmTraceWireError> {
    if bytes.len() > limits.max_total_bytes {
        return Err(VmTraceWireError::LimitExceeded {
            context: "trace bytes",
            actual: bytes.len(),
            maximum: limits.max_total_bytes,
        });
    }
    const HEADER_BYTES: usize = 14;
    if bytes.len() < HEADER_BYTES {
        return Err(VmTraceWireError::TruncatedHeader {
            actual: bytes.len(),
            minimum: HEADER_BYTES,
        });
    }
    let mut reader = Reader::new(bytes, limits);
    let magic: [u8; 4] = reader
        .take("trace magic", 4)?
        .try_into()
        .expect("fixed magic slice");
    if magic != TRACE_MAGIC {
        return Err(VmTraceWireError::InvalidMagic { actual: magic });
    }
    let version = reader.read_u16("trace version")?;
    if version != TRACE_VERSION {
        return Err(VmTraceWireError::UnsupportedVersion {
            actual: version,
            expected: TRACE_VERSION,
        });
    }
    let fingerprint = reader.read_u64("trace dictionary fingerprint")?;
    if fingerprint != TRACE_SYMBOL_FINGERPRINT {
        return Err(VmTraceWireError::DictionaryMismatch {
            actual: fingerprint,
            expected: TRACE_SYMBOL_FINGERPRINT,
        });
    }

    let scenario = reader.read_symbol("scenario")?;
    let seed = reader.read_u32("seed")?;
    let sample_every = reader.read_u32("sampleEvery")?;
    let ticks = reader.read_u32("ticks")?;
    let coverage_count = reader.read_limited_length(
        "coverage entries",
        limits.max_coverage_entries,
        LengthKind::U16,
    )?;
    let mut coverage = Vec::with_capacity(coverage_count);
    for _ in 0..coverage_count {
        coverage.push(Value::String(
            reader.read_symbol("coverage symbol")?.to_string(),
        ));
    }
    let draws = reader.read_u32("draws")?;
    let draw_digest = reader.read_u32("drawDigest")?;
    let frame_count = reader.read_limited_length("frames", limits.max_frames, LengthKind::U16)?;
    let mut frames = Vec::with_capacity(frame_count);
    for _ in 0..frame_count {
        frames.push(reader.read_frame()?);
    }
    reader.finish()?;

    Ok(json!({
        "scenario": scenario,
        "seed": seed,
        "sampleEvery": sample_every,
        "ticks": ticks,
        "coverage": coverage,
        "draws": draws,
        "drawDigest": hex32(draw_digest),
        "frames": frames,
    }))
}

#[derive(Clone, Copy)]
enum LengthKind {
    U16,
    U32,
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
    limits: VmTraceDecodeLimits,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], limits: VmTraceDecodeLimits) -> Self {
        Self {
            bytes,
            offset: 0,
            limits,
        }
    }

    fn read_frame(&mut self) -> Result<Value, VmTraceWireError> {
        let tick = self.read_u64("frame.tick")?;
        safe_unsigned("frame.tick", tick)?;
        let time_value = self.read_value(0)?;
        if !matches!(time_value, TraceValue::Number(_)) {
            return Err(VmTraceWireError::ExpectedNumber {
                context: "frame.time",
            });
        }
        let time = canonical(&time_value, false);
        let next_id = self.read_u64("frame.nextId")?;
        safe_unsigned("frame.nextId", next_id)?;
        let label_id = self.read_u16("frame.label")?;
        let label = if label_id == 0 {
            None
        } else {
            Some(resolve_symbol(label_id)?.to_string())
        };
        let full = self.read_bool("frame.full")?;
        let rng_draws = self.read_u32("frame.rng.draws")?;
        let rng_digest = self.read_u32("frame.rng.digest")?;
        let players = self.read_value(0)?;
        require_array(&players, "frame.players")?;
        let entities = self.read_value(0)?;
        require_array(&entities, "frame.entities")?;
        // Parity JSON retains only the ordered event-window digest. Keeping that
        // digest on the wire avoids pretending a finite symbol dictionary can
        // encode arbitrary event text while preserving the authoritative value.
        let event_digest = self.read_u32("frame.events")?;

        let players = canonical(&players, true);
        let entities = canonical(&entities, true);
        let mut state = Map::new();
        // Insertion is lexicographic even if serde_json is built with preserve_order.
        state.insert("entities".to_string(), entities.clone());
        state.insert("players".to_string(), players.clone());
        let state_digest = digest_json(&Value::Object(state));

        let mut frame = Map::new();
        frame.insert("tick".to_string(), json!(tick));
        frame.insert("time".to_string(), time);
        frame.insert("nextId".to_string(), json!(next_id));
        frame.insert("state".to_string(), Value::String(state_digest));
        frame.insert("events".to_string(), Value::String(hex32(event_digest)));
        frame.insert(
            "rng".to_string(),
            json!({ "draws": rng_draws, "digest": hex32(rng_digest) }),
        );
        if let Some(label) = label {
            frame.insert("label".to_string(), Value::String(label));
        }
        if full {
            frame.insert("players".to_string(), players);
            frame.insert("entities".to_string(), entities);
        }
        Ok(Value::Object(frame))
    }

    fn read_value(&mut self, depth: usize) -> Result<TraceValue, VmTraceWireError> {
        if depth > self.limits.max_value_depth {
            return Err(VmTraceWireError::LimitExceeded {
                context: "value depth",
                actual: depth,
                maximum: self.limits.max_value_depth,
            });
        }
        let tag = self.read_u8("value tag")?;
        match tag {
            TAG_NULL => Ok(TraceValue::Null),
            TAG_FALSE => Ok(TraceValue::Bool(false)),
            TAG_TRUE => Ok(TraceValue::Bool(true)),
            TAG_UNSIGNED => {
                let value = self.read_u64("unsigned value")?;
                safe_unsigned("unsigned value", value)?;
                Ok(TraceValue::Number(value as f64))
            }
            TAG_SIGNED_MAGNITUDE => {
                let negative = self.read_bool("signed value sign")?;
                let magnitude = self.read_u64("signed value magnitude")?;
                safe_unsigned("signed value", magnitude)?;
                let value = magnitude as f64;
                Ok(TraceValue::Number(if negative && magnitude != 0 {
                    -value
                } else {
                    value
                }))
            }
            TAG_FIXED6 => {
                let negative = self.read_bool("fixed6 sign")?;
                let magnitude = self.read_u64("fixed6 magnitude")?;
                safe_unsigned("fixed6 value", magnitude)?;
                let value = magnitude as f64 / 1_000_000.0;
                Ok(TraceValue::Number(if negative && magnitude != 0 {
                    -value
                } else {
                    value
                }))
            }
            TAG_POSITIVE_INFINITY => Ok(TraceValue::Number(f64::INFINITY)),
            TAG_NEGATIVE_INFINITY => Ok(TraceValue::Number(f64::NEG_INFINITY)),
            TAG_NAN => Ok(TraceValue::Number(f64::NAN)),
            TAG_SYMBOL => Ok(TraceValue::String(
                self.read_symbol("string symbol")?.to_string(),
            )),
            TAG_ARRAY => {
                let length = self.read_limited_length(
                    "array entries",
                    self.limits.max_collection_entries,
                    LengthKind::U32,
                )?;
                let mut values = Vec::with_capacity(length);
                for _ in 0..length {
                    values.push(self.read_value(depth + 1)?);
                }
                Ok(TraceValue::Array(values))
            }
            TAG_OBJECT => {
                let length = self.read_limited_length(
                    "object entries",
                    self.limits.max_collection_entries,
                    LengthKind::U32,
                )?;
                let mut values = BTreeMap::new();
                for _ in 0..length {
                    let symbol = self.read_u16("object key")?;
                    let key = resolve_symbol(symbol)?.to_string();
                    let value = self.read_value(depth + 1)?;
                    if values.insert(key, value).is_some() {
                        return Err(VmTraceWireError::DuplicateObjectKey { symbol });
                    }
                }
                Ok(TraceValue::Object(values))
            }
            unknown => Err(VmTraceWireError::UnknownValueTag(unknown)),
        }
    }

    fn read_symbol(&mut self, context: &'static str) -> Result<&'static str, VmTraceWireError> {
        let id = self.read_u16(context)?;
        resolve_symbol(id)
    }

    fn read_limited_length(
        &mut self,
        context: &'static str,
        maximum: usize,
        kind: LengthKind,
    ) -> Result<usize, VmTraceWireError> {
        let actual = match kind {
            LengthKind::U16 => usize::from(self.read_u16(context)?),
            LengthKind::U32 => self.read_u32(context)? as usize,
        };
        if actual > maximum {
            return Err(VmTraceWireError::LimitExceeded {
                context,
                actual,
                maximum,
            });
        }
        Ok(actual)
    }

    fn read_bool(&mut self, context: &'static str) -> Result<bool, VmTraceWireError> {
        match self.read_u8(context)? {
            0 => Ok(false),
            1 => Ok(true),
            invalid => Err(VmTraceWireError::InvalidBoolean(invalid)),
        }
    }

    fn read_u8(&mut self, context: &'static str) -> Result<u8, VmTraceWireError> {
        Ok(self.take(context, 1)?[0])
    }

    fn read_u16(&mut self, context: &'static str) -> Result<u16, VmTraceWireError> {
        Ok(u16::from_le_bytes(
            self.take(context, 2)?.try_into().expect("fixed u16 slice"),
        ))
    }

    fn read_u32(&mut self, context: &'static str) -> Result<u32, VmTraceWireError> {
        Ok(u32::from_le_bytes(
            self.take(context, 4)?.try_into().expect("fixed u32 slice"),
        ))
    }

    fn read_u64(&mut self, context: &'static str) -> Result<u64, VmTraceWireError> {
        Ok(u64::from_le_bytes(
            self.take(context, 8)?.try_into().expect("fixed u64 slice"),
        ))
    }

    fn take(&mut self, context: &'static str, length: usize) -> Result<&'a [u8], VmTraceWireError> {
        let remaining = self.bytes.len().saturating_sub(self.offset);
        if remaining < length {
            return Err(VmTraceWireError::TruncatedPayload {
                context,
                needed: length,
                remaining,
            });
        }
        let start = self.offset;
        self.offset += length;
        Ok(&self.bytes[start..self.offset])
    }

    fn finish(self) -> Result<(), VmTraceWireError> {
        let remaining = self.bytes.len() - self.offset;
        if remaining == 0 {
            Ok(())
        } else {
            Err(VmTraceWireError::TrailingBytes { remaining })
        }
    }
}

fn require_array(value: &TraceValue, context: &'static str) -> Result<(), VmTraceWireError> {
    if matches!(value, TraceValue::Array(_)) {
        Ok(())
    } else {
        Err(VmTraceWireError::ExpectedArray { context })
    }
}

fn resolve_symbol(id: u16) -> Result<&'static str, VmTraceWireError> {
    resolve_trace_symbol(id).ok_or(VmTraceWireError::UnknownSymbol(id))
}

fn safe_unsigned(context: &'static str, magnitude: u64) -> Result<(), VmTraceWireError> {
    if magnitude <= JS_MAX_SAFE_INTEGER {
        Ok(())
    } else {
        Err(VmTraceWireError::NumberOutOfRange { context, magnitude })
    }
}

fn digest_json(value: &Value) -> String {
    fnv1a_hex(&serde_json::to_string(value).expect("canonical trace value must serialize"))
}

fn hex32(value: u32) -> String {
    format!("{value:08x}")
}
