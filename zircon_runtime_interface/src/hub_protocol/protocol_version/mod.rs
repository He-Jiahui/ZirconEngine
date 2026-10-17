#[path = "serde.rs"]
mod serde;

pub use serde::HubProtocolVersionV1;

/// The only file-mailbox protocol revision currently supported by Hub and Editor.
/// 文件协议当前唯一接受的版本号；wire 形状变更时须同步升级。
pub const HUB_PROTOCOL_VERSION_V1: u32 = 1;
