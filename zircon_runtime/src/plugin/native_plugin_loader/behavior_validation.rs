//! 入口报告的行为健康投影；版本/schema 与回调存在性形成诊断，消费方再决定加载或迁移策略。
//! 它不执行外来回调，也不证明外来指针、状态内容或注册表内容的有效性。

mod callbacks;
mod diagnostics;
mod report;
mod schema;

#[cfg(test)]
#[path = "behavior_validation/tests/cases.rs"]
mod tests;

pub use report::{NativePluginBehaviorHealth, NativePluginBehaviorValidationReport};
pub(super) use schema::{
    ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V4, ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3,
};
