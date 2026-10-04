//! 将选择意图接入槽位变更；每次调用重新解析目标，场景负载与槽位身份的约束由显式 ID 接口负责。
mod metadata;
mod remove;
mod rename;
mod touch;
