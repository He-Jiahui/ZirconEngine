//! 图诊断覆盖计划、执行和后处理节点；分发共享 RenderStats 帧边界，供比较两阶段差异。
#[path = "../../graph/execution.rs"]
mod execution;
#[path = "../../graph/execution_resources.rs"]
mod execution_resources;
#[path = "frame.rs"]
mod frame;
#[path = "../../graph/materialization.rs"]
mod materialization;
#[path = "post.rs"]
mod post_process;

use crate::core::framework::render::RenderStats;

use super::DiagnosticStore;

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    frame::record(store, stats);
    post_process::record(store, stats);
}
