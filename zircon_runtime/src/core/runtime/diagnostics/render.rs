use crate::core::framework::render::RenderStats;

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
