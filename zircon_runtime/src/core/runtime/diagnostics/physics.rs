use crate::core::framework::physics::PhysicsBackendStatus;

use super::FrameDiagnostics;

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

impl FrameDiagnostics for RuntimePhysicsDiagnostics {
    fn diagnostics_domain(&self) -> &'static str {
        "physics"
    }

    fn diagnostics_available(&self) -> bool {
        self.available
    }

    fn diagnostics_error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}
