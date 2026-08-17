use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

use serde::de::{DeserializeOwned, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::Serialize;
use zircon_runtime_interface::{ZrByteSlice, ZrByteSliceError, ZrRuntimePayloadLimitV1};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum BoundedJsonError {
    Slice(ZrByteSliceError),
    Empty,
    EncodedBytes { observed: usize, limit: usize },
    Items { observed: usize, limit: usize },
    NestingDepth { observed: usize, limit: usize },
    ProcessingTime { limit_micros: u64 },
    Json(String),
}

impl BoundedJsonError {
    pub(super) const fn is_limit_exceeded(&self) -> bool {
        matches!(
            self,
            Self::Slice(ZrByteSliceError::LengthExceedsLimit { .. })
                | Self::EncodedBytes { .. }
                | Self::Items { .. }
                | Self::NestingDepth { .. }
                | Self::ProcessingTime { .. }
        )
    }
}

impl std::fmt::Display for BoundedJsonError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Slice(error) => write!(formatter, "invalid byte slice: {error:?}"),
            Self::Empty => formatter.write_str("empty JSON payload is not allowed"),
            Self::EncodedBytes { observed, limit } => {
                write!(
                    formatter,
                    "JSON payload encoded {observed} bytes; maximum is {limit}"
                )
            }
            Self::Items { observed, limit } => {
                write!(
                    formatter,
                    "JSON payload contains {observed} items; maximum is {limit}"
                )
            }
            Self::NestingDepth { observed, limit } => {
                write!(
                    formatter,
                    "JSON payload nesting depth is {observed}; maximum is {limit}"
                )
            }
            Self::ProcessingTime { limit_micros } => {
                write!(
                    formatter,
                    "JSON processing exceeded {limit_micros} microseconds"
                )
            }
            Self::Json(message) => formatter.write_str(message),
        }
    }
}

pub(super) unsafe fn checked_bytes<'a>(
    input: ZrByteSlice,
    limit: ZrRuntimePayloadLimitV1,
) -> Result<&'a [u8], BoundedJsonError> {
    unsafe { input.checked_slice(limit.max_encoded_bytes) }.map_err(BoundedJsonError::Slice)
}

pub(super) unsafe fn decode<T>(
    input: ZrByteSlice,
    limit: ZrRuntimePayloadLimitV1,
    item_count: impl FnOnce(&T) -> usize,
) -> Result<T, BoundedJsonError>
where
    T: DeserializeOwned,
{
    let bytes = unsafe { checked_bytes(input, limit) }?;
    if bytes.is_empty() && !limit.allow_empty {
        return Err(BoundedJsonError::Empty);
    }
    let deadline = ProcessingDeadline::new(limit.max_processing_time_micros);
    let mut nesting = JsonNestingTracker::default();
    for chunk in bytes.chunks(4 * 1024) {
        deadline.check()?;
        nesting.inspect(chunk, limit.max_nesting_depth)?;
    }
    // `max_items` counts typed rows, events, or requests. The wire graph also contains
    // envelopes and scalar fields, so cap the allocation-free syntax pass by the byte
    // ceiling and apply the exact business-item policy after typed deserialization.
    preflight_json_graph(bytes, limit.max_encoded_bytes.saturating_add(1), deadline)?;
    let reader = DeadlineReader::new(bytes, deadline);
    let value = serde_json::from_reader(reader).map_err(|error| {
        if deadline.exceeded() {
            BoundedJsonError::ProcessingTime {
                limit_micros: limit.max_processing_time_micros,
            }
        } else {
            BoundedJsonError::Json(error.to_string())
        }
    })?;
    deadline.check()?;
    let observed = item_count(&value);
    deadline.check()?;
    if observed > limit.max_items {
        return Err(BoundedJsonError::Items {
            observed,
            limit: limit.max_items,
        });
    }
    Ok(value)
}

pub(super) fn encode<T: Serialize + ?Sized>(
    value: &T,
    limit: ZrRuntimePayloadLimitV1,
    item_count: impl FnOnce() -> usize,
) -> Result<Vec<u8>, BoundedJsonError> {
    let mut writer = BoundedJsonWriter::new(limit);
    let item_count = item_count();
    writer.deadline.check()?;
    if item_count > limit.max_items {
        return Err(BoundedJsonError::Items {
            observed: item_count,
            limit: limit.max_items,
        });
    }
    let result = serde_json::to_writer(&mut writer, value);
    writer.finish(result)
}

pub(super) fn validate<T: Serialize + ?Sized>(
    value: &T,
    limit: ZrRuntimePayloadLimitV1,
    item_count: impl FnOnce() -> usize,
) -> Result<(), BoundedJsonError> {
    let observed = item_count();
    if observed > limit.max_items {
        return Err(BoundedJsonError::Items {
            observed,
            limit: limit.max_items,
        });
    }
    let mut writer = BoundedJsonCountingWriter::new(limit);
    let result = serde_json::to_writer(&mut writer, value);
    writer.finish(result)
}

fn preflight_json_graph(
    bytes: &[u8],
    max_items: usize,
    deadline: ProcessingDeadline,
) -> Result<(), BoundedJsonError> {
    let mut counter = JsonItemCounter::new(max_items);
    let reader = DeadlineReader::new(bytes, deadline);
    let mut deserializer = serde_json::Deserializer::from_reader(reader);
    let result = JsonItemSeed {
        counter: &mut counter,
    }
    .deserialize(&mut deserializer)
    .and_then(|()| deserializer.end());
    if let Some(observed) = counter.overflow_observed {
        return Err(BoundedJsonError::Items {
            observed,
            limit: max_items,
        });
    }
    result.map_err(|error| {
        if deadline.exceeded() {
            BoundedJsonError::ProcessingTime {
                limit_micros: deadline.limit.as_micros().try_into().unwrap_or(u64::MAX),
            }
        } else {
            BoundedJsonError::Json(error.to_string())
        }
    })?;
    deadline.check()
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

struct BoundedJsonCountingWriter {
    count: usize,
    limit: ZrRuntimePayloadLimitV1,
    deadline: ProcessingDeadline,
    nesting: JsonNestingTracker,
    failure: Option<BoundedJsonError>,
}

impl BoundedJsonCountingWriter {
    fn new(limit: ZrRuntimePayloadLimitV1) -> Self {
        Self {
            count: 0,
            limit,
            deadline: ProcessingDeadline::new(limit.max_processing_time_micros),
            nesting: JsonNestingTracker::default(),
            failure: None,
        }
    }

    fn finish(mut self, result: Result<(), serde_json::Error>) -> Result<(), BoundedJsonError> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        result.map_err(|error| BoundedJsonError::Json(error.to_string()))?;
        self.deadline.check()?;
        if self.count == 0 && !self.limit.allow_empty {
            return Err(BoundedJsonError::Empty);
        }
        Ok(())
    }
}

impl Write for BoundedJsonCountingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(io::Error::other(
                "bounded JSON counting writer already failed",
            ));
        }
        if let Err(error) = self.deadline.check() {
            self.failure = Some(error);
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "bounded JSON processing deadline exceeded",
            ));
        }
        let Some(encoded_len) = self.count.checked_add(bytes.len()) else {
            self.failure = Some(BoundedJsonError::EncodedBytes {
                observed: usize::MAX,
                limit: self.limit.max_encoded_bytes,
            });
            return Err(io::Error::other("bounded JSON byte count overflowed"));
        };
        if encoded_len > self.limit.max_encoded_bytes {
            self.failure = Some(BoundedJsonError::EncodedBytes {
                observed: encoded_len,
                limit: self.limit.max_encoded_bytes,
            });
            return Err(io::Error::other("bounded JSON byte limit exceeded"));
        }
        if let Err(error) = self.nesting.inspect(bytes, self.limit.max_nesting_depth) {
            self.failure = Some(error);
            return Err(io::Error::other("bounded JSON nesting depth exceeded"));
        }
        self.count = encoded_len;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) struct BoundedJsonWriter {
    bytes: Vec<u8>,
    limit: ZrRuntimePayloadLimitV1,
    deadline: ProcessingDeadline,
    nesting: JsonNestingTracker,
    failure: Option<BoundedJsonError>,
}

impl BoundedJsonWriter {
    pub(super) fn new(limit: ZrRuntimePayloadLimitV1) -> Self {
        Self::with_capacity(limit, 0)
    }

    pub(super) fn with_capacity(limit: ZrRuntimePayloadLimitV1, capacity: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity.min(limit.max_encoded_bytes)),
            limit,
            deadline: ProcessingDeadline::new(limit.max_processing_time_micros),
            nesting: JsonNestingTracker::default(),
            failure: None,
        }
    }

    pub(super) fn finish(
        mut self,
        result: Result<(), serde_json::Error>,
    ) -> Result<Vec<u8>, BoundedJsonError> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        result.map_err(|error| BoundedJsonError::Json(error.to_string()))?;
        self.deadline.check()?;
        if self.bytes.is_empty() && !self.limit.allow_empty {
            return Err(BoundedJsonError::Empty);
        }
        Ok(self.bytes)
    }

    pub(super) fn finish_io_result(
        mut self,
        result: io::Result<()>,
    ) -> Result<Vec<u8>, BoundedJsonError> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        result.map_err(|error| BoundedJsonError::Json(error.to_string()))?;
        self.deadline.check()?;
        if self.bytes.is_empty() && !self.limit.allow_empty {
            return Err(BoundedJsonError::Empty);
        }
        Ok(self.bytes)
    }
}

impl Write for BoundedJsonWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(io::Error::other("bounded JSON writer already failed"));
        }
        if let Err(error) = self.deadline.check() {
            self.failure = Some(error);
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "bounded JSON processing deadline exceeded",
            ));
        }
        let Some(encoded_len) = self.bytes.len().checked_add(bytes.len()) else {
            let error = BoundedJsonError::EncodedBytes {
                observed: usize::MAX,
                limit: self.limit.max_encoded_bytes,
            };
            self.failure = Some(error);
            return Err(io::Error::other("bounded JSON byte count overflowed"));
        };
        if encoded_len > self.limit.max_encoded_bytes {
            self.failure = Some(BoundedJsonError::EncodedBytes {
                observed: encoded_len,
                limit: self.limit.max_encoded_bytes,
            });
            return Err(io::Error::other("bounded JSON byte limit exceeded"));
        }
        if let Err(error) = self.nesting.inspect(bytes, self.limit.max_nesting_depth) {
            self.failure = Some(error);
            return Err(io::Error::other("bounded JSON nesting depth exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Default)]
struct JsonNestingTracker {
    depth: usize,
    in_string: bool,
    escaped: bool,
}

impl JsonNestingTracker {
    fn inspect(&mut self, bytes: &[u8], limit: usize) -> Result<(), BoundedJsonError> {
        for byte in bytes {
            if self.in_string {
                if self.escaped {
                    self.escaped = false;
                } else if *byte == b'\\' {
                    self.escaped = true;
                } else if *byte == b'"' {
                    self.in_string = false;
                }
                continue;
            }
            match *byte {
                b'"' => self.in_string = true,
                b'{' | b'[' => {
                    self.depth = self.depth.saturating_add(1);
                    if self.depth > limit {
                        return Err(BoundedJsonError::NestingDepth {
                            observed: self.depth,
                            limit,
                        });
                    }
                }
                b'}' | b']' => self.depth = self.depth.saturating_sub(1),
                _ => {}
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct ProcessingDeadline {
    started: Instant,
    limit: Duration,
}

impl ProcessingDeadline {
    fn new(limit_micros: u64) -> Self {
        Self {
            started: Instant::now(),
            limit: Duration::from_micros(limit_micros),
        }
    }

    fn exceeded(self) -> bool {
        self.started.elapsed() > self.limit
    }

    fn check(self) -> Result<(), BoundedJsonError> {
        if self.exceeded() {
            return Err(BoundedJsonError::ProcessingTime {
                limit_micros: self.limit.as_micros().try_into().unwrap_or(u64::MAX),
            });
        }
        Ok(())
    }
}

struct DeadlineReader<'a> {
    bytes: &'a [u8],
    offset: usize,
    deadline: ProcessingDeadline,
}

impl<'a> DeadlineReader<'a> {
    fn new(bytes: &'a [u8], deadline: ProcessingDeadline) -> Self {
        Self {
            bytes,
            offset: 0,
            deadline,
        }
    }
}

impl Read for DeadlineReader<'_> {
    fn read(&mut self, destination: &mut [u8]) -> io::Result<usize> {
        self.deadline.check().map_err(|_| {
            io::Error::new(
                io::ErrorKind::TimedOut,
                "bounded JSON processing deadline exceeded",
            )
        })?;
        if self.offset == self.bytes.len() {
            return Ok(0);
        }
        let count = destination
            .len()
            .min(4 * 1024)
            .min(self.bytes.len() - self.offset);
        destination[..count].copy_from_slice(&self.bytes[self.offset..self.offset + count]);
        self.offset += count;
        Ok(count)
    }
}

pub(super) fn json_value_item_count(value: &serde_json::Value) -> usize {
    let mut count = 0_usize;
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        count = count.saturating_add(1);
        match value {
            serde_json::Value::Array(values) => pending.extend(values),
            serde_json::Value::Object(values) => pending.extend(values.values()),
            _ => {}
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use serde::de::IgnoredAny;
    use serde::Deserialize;

    use super::*;

    static BUSINESS_DESERIALIZE_CALLS: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    struct BusinessDeserializeProbe;

    impl<'de> Deserialize<'de> for BusinessDeserializeProbe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            BUSINESS_DESERIALIZE_CALLS.fetch_add(1, Ordering::SeqCst);
            IgnoredAny::deserialize(deserializer)?;
            Ok(Self)
        }
    }

    fn depth_limit(max_nesting_depth: usize) -> ZrRuntimePayloadLimitV1 {
        ZrRuntimePayloadLimitV1 {
            max_encoded_bytes: 4 * 1024,
            max_items: 4 * 1024,
            max_nesting_depth,
            max_processing_time_micros: 100_000,
            allow_empty: false,
        }
    }

    #[test]
    fn decode_reports_nesting_as_a_payload_limit() {
        let encoded = format!("{}0{}", "[".repeat(5), "]".repeat(5));
        let input = ZrByteSlice {
            data: encoded.as_ptr(),
            len: encoded.len(),
        };

        let error =
            unsafe { decode::<serde_json::Value>(input, depth_limit(4), json_value_item_count) }
                .expect_err(
                    "inbound JSON above the declared nesting depth must be rejected as a limit",
                );

        assert_eq!(
            error,
            BoundedJsonError::NestingDepth {
                observed: 5,
                limit: 4
            }
        );
        assert!(error.is_limit_exceeded());
    }

    #[test]
    fn encode_reports_nesting_as_a_payload_limit() {
        let value: serde_json::Value =
            serde_json::from_str(&format!("{}0{}", "[".repeat(5), "]".repeat(5))).unwrap();

        let error = encode(&value, depth_limit(4), || json_value_item_count(&value))
            .expect_err("outbound JSON above the declared nesting depth must be rejected");

        assert_eq!(
            error,
            BoundedJsonError::NestingDepth {
                observed: 5,
                limit: 4
            }
        );
        assert!(error.is_limit_exceeded());
    }

    #[test]
    fn validate_reports_limits_without_materializing_encoded_bytes() {
        let value = serde_json::json!({"payload": "too large"});
        let limit = ZrRuntimePayloadLimitV1 {
            max_encoded_bytes: 8,
            ..depth_limit(8)
        };

        let error = validate(&value, limit, || 1)
            .expect_err("counting validation must reject the oversized payload");

        assert!(matches!(
            error,
            BoundedJsonError::EncodedBytes { limit: 8, .. }
        ));
    }

    #[test]
    fn nesting_tracker_ignores_delimiters_inside_split_escaped_strings() {
        let mut tracker = JsonNestingTracker::default();
        tracker.inspect(br#"{"value":"\"#, 1).unwrap();
        tracker.inspect(br#""[{}]"}"#, 1).unwrap();
        assert_eq!(tracker.depth, 0);
    }

    #[test]
    fn decode_applies_the_item_limit_to_business_items_not_json_nodes() {
        BUSINESS_DESERIALIZE_CALLS.store(0, Ordering::SeqCst);
        let encoded = br#"[0,1,2]"#;
        let input = ZrByteSlice {
            data: encoded.as_ptr(),
            len: encoded.len(),
        };
        let limit = ZrRuntimePayloadLimitV1 {
            max_items: 2,
            ..depth_limit(8)
        };

        unsafe { decode::<BusinessDeserializeProbe>(input, limit, |_| 1) }
            .expect("one business item must fit even when its JSON representation has more nodes");

        assert_eq!(BUSINESS_DESERIALIZE_CALLS.load(Ordering::SeqCst), 1);
    }
}
