use super::{
    DiagnosticStoreSnapshot, FrameDiagnostics, FrameDiagnosticsStatus, RuntimeAnimationDiagnostics,
    RuntimePhysicsDiagnostics, RuntimeRenderDiagnostics,
};
use zircon_runtime_interface::ProfileSnapshot;

/// 一次运行时诊断采集的跨域结果；服务状态、数值历史和性能时间线有各自的生命周期。
/// runtime_diagnostics::collect_runtime_diagnostics 填充它，编辑器面板按域消费缺失值。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeDiagnosticsSnapshot {
    pub render: RuntimeRenderDiagnostics,
    pub physics: RuntimePhysicsDiagnostics,
    pub animation: RuntimeAnimationDiagnostics,
    pub store: DiagnosticStoreSnapshot,
    pub profile: ProfileSnapshot,
}

impl RuntimeDiagnosticsSnapshot {
    /// 只汇总有服务可用性语义的三个帧域；数值存储和性能录制不参与该状态列表。
    pub fn frame_diagnostics_statuses(&self) -> [FrameDiagnosticsStatus<'_>; 3] {
        [
            self.render.frame_diagnostics_status(),
            self.physics.frame_diagnostics_status(),
            self.animation.frame_diagnostics_status(),
        ]
    }
}

#[cfg(test)]
#[path = "tests/snapshot.rs"]
mod tests;
