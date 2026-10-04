//! 固定 RPC feature 能力名，供握手 required_features 与宿主 feature 选择共享。
//! 能力位只表达协议兼容，不构成连接身份或玩家认证。

pub use crate::feature::NET_RPC_FEATURE_CAPABILITY;

pub const RUNTIME_CAPABILITIES: &[&str] = &[NET_RPC_FEATURE_CAPABILITY];
