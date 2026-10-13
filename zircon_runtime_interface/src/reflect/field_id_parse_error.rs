#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
/// 字段稳定 ID 的 UUID 文本解析或非 nil 校验错误。
pub enum ReflectFieldIdParseError {
    /// 输入不能解析为 UUID；保留底层 UUID 解析原因。
    #[error("reflect field ID is not a UUID: {source}")]
    InvalidUuid { source: uuid::Error },
    /// 输入是全零 UUID，不能作为稳定字段身份。
    #[error("reflect field ID must not be nil")]
    Nil,
}
