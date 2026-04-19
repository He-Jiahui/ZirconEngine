//! Physics framework contracts (backend selection, materials, simulation mode).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicsCombineRule {
    #[default]
    Average,
    Minimum,
    Maximum,
    Multiply,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PhysicsMaterialMetadata {
    pub static_friction: f32,
    pub dynamic_friction: f32,
    pub restitution: f32,
    pub friction_combine: PhysicsCombineRule,
    pub restitution_combine: PhysicsCombineRule,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicsSimulationMode {
    Disabled,
    Simulate,
    QueryOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsSettings {
    pub backend: String,
    pub simulation_mode: PhysicsSimulationMode,
    pub fixed_hz: u32,
    pub max_substeps: u32,
    pub layer_names: Vec<String>,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            backend: "unconfigured".to_string(),
            simulation_mode: PhysicsSimulationMode::Disabled,
            fixed_hz: 60,
            max_substeps: 4,
            layer_names: vec!["default".to_string()],
        }
    }
}

pub trait PhysicsManager: Send + Sync {
    fn backend_name(&self) -> String;
    fn settings(&self) -> PhysicsSettings;
    fn default_material(&self) -> PhysicsMaterialMetadata;
}
