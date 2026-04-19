use crate::core::framework::physics::{PhysicsMaterialMetadata, PhysicsSettings, PhysicsSimulationMode};

use super::PhysicsInterface;

pub const JOLT_ENABLED: bool = cfg!(feature = "jolt");

#[derive(Clone, Debug, Default)]
pub struct PhysicsDriver;

#[derive(Clone, Debug)]
pub struct DefaultPhysicsManager {
    settings: PhysicsSettings,
    default_material: PhysicsMaterialMetadata,
}

impl Default for DefaultPhysicsManager {
    fn default() -> Self {
        Self {
            settings: PhysicsSettings {
                backend: if JOLT_ENABLED {
                    "jolt".to_string()
                } else {
                    "unconfigured".to_string()
                },
                simulation_mode: if JOLT_ENABLED {
                    PhysicsSimulationMode::Simulate
                } else {
                    PhysicsSimulationMode::Disabled
                },
                ..PhysicsSettings::default()
            },
            default_material: PhysicsMaterialMetadata::default(),
        }
    }
}

impl crate::core::framework::physics::PhysicsManager for DefaultPhysicsManager {
    fn backend_name(&self) -> String {
        self.settings.backend.clone()
    }

    fn settings(&self) -> PhysicsSettings {
        self.settings.clone()
    }

    fn default_material(&self) -> PhysicsMaterialMetadata {
        self.default_material.clone()
    }
}

impl PhysicsInterface for DefaultPhysicsManager {}
