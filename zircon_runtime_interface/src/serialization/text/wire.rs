/// Maximum encoded size accepted or emitted by the authoring text wire.
/// 此固定上限仅用于版本化文本文档的读取和写入；原始 `write_canonical_text_to`
/// 使用调用方提供的 `SerializationBudget`，不套用此值。
pub(in crate::serialization) const MAX_TEXT_DOCUMENT_BYTES: usize = 64 * 1024 * 1024;
