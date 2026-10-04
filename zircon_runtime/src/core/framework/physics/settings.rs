use serde::{Deserialize, Serialize};

use super::PhysicsSimulationMode;

/// 用于配置后端选择、时钟和碰撞层；插件可持久化设置并据此决定是否启用模拟。
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

// 基础默认值保持物理禁用；插件仅在明确拥有可用后端时覆盖后端名和模拟模式。
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
