use serde::{Deserialize, Serialize};

use super::PhysicsSimulationMode;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsSettings {
    pub backend: String,
    pub simulation_mode: PhysicsSimulationMode,
    pub fixed_hz: u32,
    pub max_substeps: u32,
    pub layer_names: Vec<String>,
    pub group_names: Vec<String>,
    pub collision_matrix: Vec<u64>,
    pub solver_groups: Vec<String>,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            backend: "unconfigured".to_string(),
            simulation_mode: PhysicsSimulationMode::Disabled,
            fixed_hz: 60,
            max_substeps: 4,
            layer_names: vec!["default".to_string()],
            group_names: vec!["default".to_string()],
            collision_matrix: vec![0b1],
            solver_groups: vec!["default".to_string()],
        }
    }
}
