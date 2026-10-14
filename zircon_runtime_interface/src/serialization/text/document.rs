use serde::Serialize;

use super::envelope::TextEnvelope;

/// Top-level text document which keeps the current payload borrowed through encoding.
/// `serde` 将 `envelope` 序列化为顶层 `$zircon` 字段；此包装只描述线格式，
/// 不自行验证封套或 payload，读取侧会在识别格式后执行相应校验。
#[derive(Serialize)]
pub(in crate::serialization) struct TextDocument<'a, T: ?Sized> {
    /// 版本化文本格式保留的顶层封套键。
    #[serde(rename = "$zircon")]
    pub(in crate::serialization) envelope: TextEnvelope<'a, T>,
}
