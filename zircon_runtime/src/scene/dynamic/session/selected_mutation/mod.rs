//! 将选择意图接入槽位变更；每次调用重新解析目标，场景负载与槽位身份的约束由显式 ID 接口负责。
#[path = "metadata/mod.rs"]
mod metadata;
#[path = "remove/mod.rs"]
mod remove;
#[path = "rename/mod.rs"]
mod rename;
#[path = "touch/mod.rs"]
mod touch;
