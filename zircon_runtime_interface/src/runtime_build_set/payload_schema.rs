use super::ZrRuntimeDigestV1;

const CURRENT_PAYLOAD_SCHEMA_SET_SOURCE: &[u8] = include_bytes!("payload_schema_set_v1.json");

/// Digest of the current internal payload schema-set source.
///
/// V1 records the existing entry-local UTF-8 JSON boundary. It intentionally
/// does not claim that those entry payloads already share a public envelope.
pub fn current_runtime_payload_schema_set_digest() -> ZrRuntimeDigestV1 {
    ZrRuntimeDigestV1::sha256(CURRENT_PAYLOAD_SCHEMA_SET_SOURCE)
}
