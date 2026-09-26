/// Always-on tooling projection of an optional Physics backend contract.
/// 即使物理契约没有编入当前构建，工具侧仍可使用此类型承载后端选择和降级状态。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePhysicsBackendDiagnostics {
    pub requested_backend: String,
    pub active_backend: Option<String>,
    pub state: String,
    pub detail: Option<String>,
    pub simulation_mode: String,
    pub feature_gate: Option<String>,
}
