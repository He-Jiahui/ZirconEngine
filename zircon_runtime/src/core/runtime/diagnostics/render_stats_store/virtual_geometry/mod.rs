//! 虚拟几何诊断沿输入预算、驻留、执行和裁剪链路组织，定位降级阶段。
mod admission;
mod culling;
mod debug;
mod execution;
mod residency;

use crate::core::framework::render::RenderStats;

use super::DiagnosticStore;

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    admission::record(store, stats);
    debug::record(store, stats);
    residency::record(store, stats);
    execution::record(store, stats);
    culling::record(store, stats);
}
