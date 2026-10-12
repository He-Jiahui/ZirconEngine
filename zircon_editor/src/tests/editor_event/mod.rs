//! 编辑器事件的测试入口：分别校验服务留存、运行时派发、动画资产及协议迁移。
#[path = "animation_runtime/mod.rs"]
mod animation_runtime;
#[path = "focused_document_projection_hard_cut.rs"]
mod focused_document_projection_hard_cut;
#[path = "locator_protocol_hard_cut.rs"]
mod locator_protocol_hard_cut;
#[path = "retention.rs"]
mod retention;
#[path = "runtime/mod.rs"]
mod runtime;
#[path = "service.rs"]
mod service;
#[path = "support.rs"]
pub(crate) mod support;
