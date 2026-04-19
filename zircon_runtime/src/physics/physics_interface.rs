use crate::core::framework::physics::{PhysicsManager, PhysicsSimulationMode};

pub trait PhysicsInterface: PhysicsManager {
    fn is_enabled(&self) -> bool {
        self.settings().simulation_mode != PhysicsSimulationMode::Disabled
    }
}
