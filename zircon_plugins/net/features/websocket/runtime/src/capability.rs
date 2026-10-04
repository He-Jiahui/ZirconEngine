//! 公开 WS feature 能力名，供宿主选择后端装配和根 net 包的可选功能清单使用。

pub use crate::feature::NET_WEBSOCKET_FEATURE_CAPABILITY;

pub const RUNTIME_CAPABILITIES: &[&str] = &[NET_WEBSOCKET_FEATURE_CAPABILITY];
