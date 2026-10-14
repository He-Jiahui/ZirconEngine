use serde::Serialize;

use crate::serialization::PayloadHeader;

/// Current-version text envelope which borrows the payload while it is written.
/// 版本化写入入口负责构造 `header`，顶层 `TextDocument` 再将此结构放到保留键 `$zircon` 下。
/// 本结构只定义序列化形状；schema、版本与载荷内容的校验由调用路径负责。
#[derive(Serialize)]
pub(in crate::serialization) struct TextEnvelope<'a, T: ?Sized> {
    /// 与当前载荷对应的 schema 标识和版本元数据。
    pub(in crate::serialization) header: PayloadHeader,
    /// 在同步写出期间借用原始值，不先物化为拥有所有权的 JSON `Value`。
    pub(in crate::serialization) payload: &'a T,
}
