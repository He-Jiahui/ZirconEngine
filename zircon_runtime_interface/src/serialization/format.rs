/// Physical encoding used by a versioned payload.
/// 版本化载荷的线格式选择；读取后仍由 loader 统一检查 schema 身份和 schema 版本。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// 文本版本封套；无封套 schema 0 回退仅由显式 legacy loader 决定。
    Text,
    /// 二进制版本封套；读取要求有效 wire 前缀和头部，不采用无头 schema-zero 回退。
    Binary,
}
