mod deadline;
mod error;
mod preflight;
mod writer;

#[cfg(test)]
mod tests;

use serde::de::DeserializeOwned;
use serde::Serialize;
use zircon_runtime_interface::{ZrByteSlice, ZrRuntimePayloadLimitV1};

use deadline::{DeadlineReader, ProcessingDeadline};
pub(super) use error::BoundedJsonError;
pub(super) use preflight::json_value_item_count;
use preflight::preflight_json_graph;
pub(super) use writer::BoundedJsonWriter;
use writer::{BoundedJsonCountingWriter, JsonNestingTracker};

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
