use crate::core::framework::events::{
    EngineEvent, EngineEventPublishRejection, EventBusLimits, FrozenEngineEvent,
};
use std::io::{self, Write};

struct BoundedPayloadWriter {
    bytes: Vec<u8>,
    limit: usize,
    oversized: bool,
}
impl Write for BoundedPayloadWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let next = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .filter(|next| *next <= self.limit);
        if next.is_none() {
            self.oversized = true;
            return Err(io::Error::other(
                "event payload exceeds retained byte budget",
            ));
        }
        let next = next.expect("bounded payload length checked");
        if self.bytes.capacity() < next {
            let capacity = next
                .max(self.bytes.capacity().saturating_mul(2))
                .min(self.limit);
            self.bytes
                .try_reserve_exact(capacity - self.bytes.len())
                .map_err(io::Error::other)?;
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) fn freeze(
    event: &EngineEvent,
    limits: EventBusLimits,
) -> Result<FrozenEngineEvent, EngineEventPublishRejection> {
    if event.topic.is_empty() || event.topic.len() > limits.max_topic_name_bytes.get() {
        return Err(EngineEventPublishRejection::InvalidTopic);
    }
    let mut writer = BoundedPayloadWriter {
        bytes: Vec::new(),
        limit: limits.max_event_payload_bytes.get(),
        oversized: false,
    };
    if serde_json::to_writer(&mut writer, &event.payload).is_err() {
        return Err(if writer.oversized {
            EngineEventPublishRejection::PayloadTooLarge
        } else {
            EngineEventPublishRejection::SerializationFailed
        });
    }
    Ok(FrozenEngineEvent {
        topic: event.topic.as_str().into(),
        payload: writer.bytes.into_boxed_slice(),
    })
}
