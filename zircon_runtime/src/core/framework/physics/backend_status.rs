use serde::{Deserialize, Serialize};

use super::{PhysicsBackendState, PhysicsSimulationMode};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PhysicsBackendStatus {
    pub requested_backend: String,
    pub active_backend: Option<String>,
    pub state: PhysicsBackendState,
    pub detail: Option<String>,
    pub simulation_mode: PhysicsSimulationMode,
    pub feature_gate: Option<String>,
}
