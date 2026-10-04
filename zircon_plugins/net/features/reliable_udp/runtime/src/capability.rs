//! 固定 reliable UDP feature 能力名，供宿主模块依赖和可选功能清单选择。
//! 能力可见不意味着根 UDP socket 已连接到该 manager。

pub use crate::feature::NET_RELIABLE_UDP_FEATURE_CAPABILITY;

pub const RUNTIME_CAPABILITIES: &[&str] = &[NET_RELIABLE_UDP_FEATURE_CAPABILITY];
