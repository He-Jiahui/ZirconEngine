use serde::{Deserialize, Serialize};

use super::{PhysicsBackendState, PhysicsSimulationMode};

/// 后端选择的诊断快照；分别保留请求名和活动名，以呈现配置降级或运行错误。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PhysicsBackendStatus {
    pub requested_backend: String,
    pub active_backend: Option<String>,
    pub state: PhysicsBackendState,
    pub detail: Option<String>,
    pub simulation_mode: PhysicsSimulationMode,
    pub feature_gate: Option<String>,
}
