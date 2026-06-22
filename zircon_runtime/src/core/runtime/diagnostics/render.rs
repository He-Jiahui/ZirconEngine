use crate::core::framework::render::RenderStats;

use super::FrameDiagnostics;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeRenderDiagnostics {
    pub available: bool,
    pub stats: Option<RenderStats>,
    pub virtual_geometry_debug_available: bool,
    pub error: Option<String>,
}

impl RuntimeRenderDiagnostics {
    pub fn unavailable(error: impl Into<String>) -> Self {
        Self {
            available: false,
            stats: None,
            virtual_geometry_debug_available: false,
            error: Some(error.into()),
        }
    }
}

impl FrameDiagnostics for RuntimeRenderDiagnostics {
    fn diagnostics_domain(&self) -> &'static str {
        "render"
    }

    fn diagnostics_available(&self) -> bool {
        self.available
    }

    fn diagnostics_error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}
