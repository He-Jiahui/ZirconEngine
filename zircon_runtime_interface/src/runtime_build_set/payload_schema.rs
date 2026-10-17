use super::ZrRuntimeDigestV1;

const CURRENT_PAYLOAD_SCHEMA_SET_SOURCE: &[u8] = include_bytes!("payload_schema_set_v1.json");

/// Digest of the current internal payload schema-set source.
/// 当前内部 payload schema 集合源文件的摘要。
///
/// V1 记录现有入口本地的 UTF-8 JSON 边界；它有意不宣称这些入口共享公共信封。
///
/// V1 records the existing entry-local UTF-8 JSON boundary. It intentionally
/// does not claim that those entry payloads already share a public envelope.
pub fn current_runtime_payload_schema_set_digest() -> ZrRuntimeDigestV1 {
    ZrRuntimeDigestV1::sha256(CURRENT_PAYLOAD_SCHEMA_SET_SOURCE)
}
