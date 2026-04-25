use crate::core::framework::physics::PhysicsBackendStatus;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimePhysicsDiagnostics {
    pub available: bool,
    pub backend_name: Option<String>,
    pub backend_status: Option<PhysicsBackendStatus>,
    pub fixed_hz: Option<u32>,
    pub error: Option<String>,
}

impl RuntimePhysicsDiagnostics {
    pub fn unavailable(error: impl Into<String>) -> Self {
        Self {
            available: false,
            backend_name: None,
            backend_status: None,
            fixed_hz: None,
            error: Some(error.into()),
        }
    }
}
