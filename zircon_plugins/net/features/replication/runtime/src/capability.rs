//! 固定 replication feature 能力名，供宿主 feature 依赖解析与包清单选择。
//! 该声明不保证 World 实体或网络 session 已绑定复制状态。

pub use crate::feature::NET_REPLICATION_FEATURE_CAPABILITY;

pub const RUNTIME_CAPABILITIES: &[&str] = &[NET_REPLICATION_FEATURE_CAPABILITY];
