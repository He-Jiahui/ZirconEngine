//! Bounded JSON decoding for foreign runtime payloads.

use std::io::{self, BufReader, Read};
use std::time::{Duration, Instant};

use serde::de::{
    Deserialize, DeserializeOwned, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor,
};

use super::{
    budget::RuntimeForeignOutputPreflight, RuntimeForeignOutputBudget, RuntimeForeignOutputError,
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
    let Some(deadline) = decode_started.checked_add(budget.max_decode_time) else {
        return (
            Err(RuntimeForeignOutputError::protocol_violation(format!(
                "{operation} decode deadline exceeds the host clock range"
            ))),
            decode_started.elapsed(),
        );
    };
    // The interface item limit counts typed rows/deliveries, while the JSON graph also contains
    // envelopes and arbitrary payload values. Bound the allocation-free syntax pass by the wire
    // ceiling, then apply the exact typed item policy below.
    let json_value_limit = budget.max_encoded_bytes.saturating_add(1);
    if let Err(error) = preflight_json_graph(bytes, json_value_limit, budget, deadline, operation) {
        return (Err(error), decode_started.elapsed());
    }
    let preflight_time = decode_started.elapsed();
    if let Err(error) = budget.validate_decode_duration(preflight_time, operation) {
        return (Err(error), preflight_time);
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
    budget: RuntimeForeignOutputBudget,
    deadline: Instant,
    operation: &'static str,
) -> Result<(), RuntimeForeignOutputError> {
    let mut timed_out = false;
    let profile_files_limit = match budget.preflight {
        RuntimeForeignOutputPreflight::None => None,
        RuntimeForeignOutputPreflight::ProfileFiles => Some(budget.max_items.saturating_sub(1)),
    };
    let profile_items_limit = profile_files_limit.map(|_| budget.max_items);
    let root_context = if profile_files_limit.is_some() {
        JsonItemContext::ProfileRoot
    } else {
        JsonItemContext::General
    };
    let mut counter =
        JsonItemCounter::new(max_json_values, profile_files_limit, profile_items_limit);
    let reader = DeadlineReader::new(bytes, deadline, &mut timed_out);
    let mut deserializer = serde_json::Deserializer::from_reader(BufReader::with_capacity(
        DECODE_READER_CHUNK_BYTES,
        reader,
    ));
    let result = JsonItemSeed {
        counter: &mut counter,
        context: root_context,
        profile_item: None,
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
    if let Some(observed) = counter.profile_files_overflow_observed {
        let maximum = counter.profile_files_limit.unwrap_or_default();
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned {observed} profile files; maximum is {maximum}"
        )));
    }
    if let Some(observed) = counter.profile_items_overflow_observed {
        let maximum = counter.profile_items_limit.unwrap_or_default();
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned {observed} profile items; maximum is {maximum}"
        )));
    }
    result.map_err(|error| {
        RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} failed JSON item preflight (maximum nesting depth {RUNTIME_FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH}): {error}"
        ))
    })?;
    if let Some(maximum) = counter.profile_items_limit {
        if counter.profile_items_observed > maximum {
            return Err(RuntimeForeignOutputError::protocol_violation(format!(
                "{operation} returned {} profile items; maximum is {maximum}",
                counter.profile_items_observed
            )));
        }
    }
    Ok(())
}

struct JsonItemCounter {
    observed: usize,
    limit: usize,
    overflow_observed: Option<usize>,
    profile_files_observed: usize,
    profile_files_limit: Option<usize>,
    profile_files_overflow_observed: Option<usize>,
    profile_items_observed: usize,
    profile_items_limit: Option<usize>,
    profile_items_overflow_observed: Option<usize>,
}

impl JsonItemCounter {
    fn new(
        limit: usize,
        profile_files_limit: Option<usize>,
        profile_items_limit: Option<usize>,
    ) -> Self {
        Self {
            observed: 0,
            limit,
            overflow_observed: None,
            profile_files_observed: 0,
            profile_files_limit,
            profile_files_overflow_observed: None,
            // The response envelope is one typed item in item_count.rs.
            profile_items_observed: if profile_items_limit.is_some() { 1 } else { 0 },
            profile_items_limit,
            profile_items_overflow_observed: None,
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

    fn observe_profile_file<E: serde::de::Error>(&mut self) -> Result<(), E> {
        self.profile_files_observed = self.profile_files_observed.saturating_add(1);
        if self
            .profile_files_limit
            .is_none_or(|limit| self.profile_files_observed <= limit)
        {
            return Ok(());
        }
        self.profile_files_overflow_observed = Some(self.profile_files_observed);
        Err(E::custom("profile file limit exceeded"))
    }

    fn observe_profile_item<E: serde::de::Error>(&mut self) -> Result<(), E> {
        self.profile_items_observed = self.profile_items_observed.saturating_add(1);
        if self
            .profile_items_limit
            .is_none_or(|limit| self.profile_items_observed <= limit)
        {
            return Ok(());
        }
        self.profile_items_overflow_observed = Some(self.profile_items_observed);
        Err(E::custom("profile item limit exceeded"))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum JsonItemContext {
    General,
    ProfileRoot,
    ProfileFiles,
    ProfileSnapshot,
    ProfileRuntimeDiagnostics,
    ProfileDiagnosticSeries,
    ProfileHotspotReport,
    ProfileCounterHotspotReport,
    ProfileUiHotspotReport,
    ProfileTypedArray,
    ProfileDiagnosticSeriesArray,
    ProfileCompositionReceipt,
    ProfileHotspotHints,
}

#[derive(Clone, Copy)]
enum ProfileItemKind {
    File,
    CollectionItem,
}

struct JsonItemSeed<'a> {
    counter: &'a mut JsonItemCounter,
    context: JsonItemContext,
    profile_item: Option<ProfileItemKind>,
}

impl<'de> DeserializeSeed<'de> for JsonItemSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match self.profile_item {
            Some(ProfileItemKind::File) => {
                self.counter.observe_profile_file::<D::Error>()?;
                self.counter.observe_profile_item::<D::Error>()?;
            }
            Some(ProfileItemKind::CollectionItem) => {
                self.counter.observe_profile_item::<D::Error>()?;
            }
            None => {}
        }
        self.counter.observe::<D::Error>()?;
        deserializer.deserialize_any(JsonItemVisitor {
            counter: self.counter,
            context: self.context,
        })
    }
}

struct JsonItemVisitor<'a> {
    counter: &'a mut JsonItemCounter,
    context: JsonItemContext,
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
            context: self.context,
            profile_item: None,
        }
        .deserialize(deserializer)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        JsonItemSeed {
            counter: self.counter,
            context: self.context,
            profile_item: None,
        }
        .deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let profile_struct = self.context.is_profile_struct();
        if self.context == JsonItemContext::ProfileCompositionReceipt {
            self.counter.observe_profile_item::<A::Error>()?;
        }
        let (context, profile_item) = match self.context {
            JsonItemContext::ProfileFiles => {
                (JsonItemContext::General, Some(ProfileItemKind::File))
            }
            JsonItemContext::ProfileDiagnosticSeriesArray => (
                JsonItemContext::ProfileDiagnosticSeries,
                Some(ProfileItemKind::CollectionItem),
            ),
            JsonItemContext::ProfileTypedArray | JsonItemContext::ProfileHotspotHints => (
                JsonItemContext::General,
                Some(ProfileItemKind::CollectionItem),
            ),
            _ => (JsonItemContext::General, None),
        };
        let mut index = 0;
        while sequence
            .next_element_seed(JsonItemSeed {
                counter: self.counter,
                context: if profile_struct {
                    self.context.position_context(index)
                } else {
                    context
                },
                profile_item: if profile_struct { None } else { profile_item },
            })?
            .is_some()
        {
            index += 1;
        }
        Ok(())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        if self.context == JsonItemContext::ProfileCompositionReceipt {
            self.counter.observe_profile_item::<A::Error>()?;
        }
        if self.context.is_profile_map() {
            while let Some(key) = map.next_key::<ProfileFieldKey>()? {
                map.next_value_seed(JsonItemSeed {
                    counter: self.counter,
                    context: self.context.field_context(key),
                    profile_item: None,
                })?;
            }
        } else {
            while map.next_key::<IgnoredAny>()?.is_some() {
                map.next_value_seed(JsonItemSeed {
                    counter: self.counter,
                    context: JsonItemContext::General,
                    profile_item: None,
                })?;
            }
        }
        Ok(())
    }
}

impl JsonItemContext {
    fn is_profile_struct(self) -> bool {
        self.is_profile_map() || self == Self::ProfileCompositionReceipt
    }

    fn is_profile_map(self) -> bool {
        matches!(
            self,
            Self::ProfileRoot
                | Self::ProfileSnapshot
                | Self::ProfileRuntimeDiagnostics
                | Self::ProfileDiagnosticSeries
                | Self::ProfileHotspotReport
                | Self::ProfileCounterHotspotReport
                | Self::ProfileUiHotspotReport
        )
    }

    fn position_context(self, index: usize) -> Self {
        use JsonItemContext as Context;

        // Serde's sequence form uses declaration order in the Interface profiling DTOs.
        match (self, index) {
            (Context::ProfileRoot, 2) => Context::ProfileSnapshot,
            (Context::ProfileRoot, 3) => Context::ProfileRuntimeDiagnostics,
            (Context::ProfileRoot, 4) => Context::ProfileCompositionReceipt,
            (Context::ProfileRoot, 5) => Context::ProfileHotspotReport,
            (Context::ProfileRoot, 6) => Context::ProfileCounterHotspotReport,
            (Context::ProfileRoot, 7) => Context::ProfileUiHotspotReport,
            (Context::ProfileRoot, 9) => Context::ProfileFiles,
            (Context::ProfileSnapshot, 5..=8) => Context::ProfileTypedArray,
            (Context::ProfileRuntimeDiagnostics, 8) => Context::ProfileDiagnosticSeriesArray,
            (Context::ProfileRuntimeDiagnostics, 10) => Context::ProfileSnapshot,
            (Context::ProfileDiagnosticSeries, 2 | 7) => Context::ProfileTypedArray,
            (Context::ProfileHotspotReport, 3) => Context::ProfileTypedArray,
            (Context::ProfileHotspotReport, 4) => Context::ProfileHotspotHints,
            (Context::ProfileCounterHotspotReport, 3 | 4) => Context::ProfileTypedArray,
            (Context::ProfileUiHotspotReport, 3 | 4) => Context::ProfileTypedArray,
            _ => Context::General,
        }
    }

    fn field_context(self, field: ProfileFieldKey) -> Self {
        use JsonItemContext as Context;
        use ProfileFieldKey as Field;

        // Follow only schema-known paths; matching names in unknown fields stay general JSON.
        match (self, field) {
            (Context::ProfileRoot, Field::Files) => Context::ProfileFiles,
            (Context::ProfileRoot, Field::Snapshot) => Context::ProfileSnapshot,
            (Context::ProfileRoot, Field::RuntimeDiagnostics) => Context::ProfileRuntimeDiagnostics,
            (Context::ProfileRoot, Field::ModuleCompositionReceipt) => {
                Context::ProfileCompositionReceipt
            }
            (Context::ProfileRoot, Field::HotspotReport) => Context::ProfileHotspotReport,
            (Context::ProfileRoot, Field::CounterHotspotReport) => {
                Context::ProfileCounterHotspotReport
            }
            (Context::ProfileRoot, Field::UiHotspotReport) => Context::ProfileUiHotspotReport,
            (
                Context::ProfileSnapshot,
                Field::Frames | Field::Spans | Field::Counters | Field::RecorderRetention,
            ) => Context::ProfileTypedArray,
            (Context::ProfileRuntimeDiagnostics, Field::DiagnosticSeries) => {
                Context::ProfileDiagnosticSeriesArray
            }
            (Context::ProfileRuntimeDiagnostics, Field::Profile) => Context::ProfileSnapshot,
            (Context::ProfileDiagnosticSeries, Field::SubsystemTags | Field::History) => {
                Context::ProfileTypedArray
            }
            (Context::ProfileHotspotReport, Field::Hotspots) => Context::ProfileTypedArray,
            (Context::ProfileHotspotReport, Field::Hints) => Context::ProfileHotspotHints,
            (Context::ProfileCounterHotspotReport, Field::Counters | Field::Hints) => {
                Context::ProfileTypedArray
            }
            (Context::ProfileUiHotspotReport, Field::Scenarios | Field::Alerts) => {
                Context::ProfileTypedArray
            }
            _ => Context::General,
        }
    }
}

#[derive(Clone, Copy)]
enum ProfileFieldKey {
    Files,
    Snapshot,
    RuntimeDiagnostics,
    ModuleCompositionReceipt,
    HotspotReport,
    CounterHotspotReport,
    UiHotspotReport,
    Frames,
    Spans,
    Counters,
    RecorderRetention,
    DiagnosticSeries,
    Profile,
    SubsystemTags,
    History,
    Hotspots,
    Hints,
    Scenarios,
    Alerts,
    Other,
}

impl<'de> Deserialize<'de> for ProfileFieldKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(ProfileFieldKeyVisitor)
    }
}

struct ProfileFieldKeyVisitor;

impl Visitor<'_> for ProfileFieldKeyVisitor {
    type Value = ProfileFieldKey;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a profile response collection field")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(match value {
            "files" => ProfileFieldKey::Files,
            "snapshot" => ProfileFieldKey::Snapshot,
            "runtime_diagnostics" => ProfileFieldKey::RuntimeDiagnostics,
            "module_composition_receipt" => ProfileFieldKey::ModuleCompositionReceipt,
            "hotspot_report" => ProfileFieldKey::HotspotReport,
            "counter_hotspot_report" => ProfileFieldKey::CounterHotspotReport,
            "ui_hotspot_report" => ProfileFieldKey::UiHotspotReport,
            "frames" => ProfileFieldKey::Frames,
            "spans" => ProfileFieldKey::Spans,
            "counters" => ProfileFieldKey::Counters,
            "recorder_retention" => ProfileFieldKey::RecorderRetention,
            "diagnostic_series" => ProfileFieldKey::DiagnosticSeries,
            "profile" => ProfileFieldKey::Profile,
            "subsystem_tags" => ProfileFieldKey::SubsystemTags,
            "history" => ProfileFieldKey::History,
            "hotspots" => ProfileFieldKey::Hotspots,
            "hints" => ProfileFieldKey::Hints,
            "scenarios" => ProfileFieldKey::Scenarios,
            "alerts" => ProfileFieldKey::Alerts,
            _ => ProfileFieldKey::Other,
        })
    }

    fn visit_borrowed_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        self.visit_str(value)
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
