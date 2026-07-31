use crate::Event;

pub const FNV1A_OFFSET: u32 = 0x811c_9dc5;
const FNV1A_PRIME: u32 = 0x0100_0193;

pub fn fnv1a_bytes(bytes: &[u8]) -> u32 {
    fold_bytes(FNV1A_OFFSET, bytes)
}

pub fn event_stream_digest(events: &[Event]) -> u32 {
    let mut digest = FNV1A_OFFSET;
    for event in events {
        digest = fold_bytes(digest, &event.event_id.to_le_bytes());
        digest = fold_bytes(digest, &event.sequence.to_le_bytes());
        digest = fold_bytes(digest, &(event.payload.len() as u64).to_le_bytes());
        digest = fold_bytes(digest, &event.payload);
    }
    digest
}

fn fold_bytes(mut digest: u32, bytes: &[u8]) -> u32 {
    for byte in bytes {
        digest ^= u32::from(*byte);
        digest = digest.wrapping_mul(FNV1A_PRIME);
    }
    digest
}
