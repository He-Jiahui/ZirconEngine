use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderingBackendInfo {
    pub backend_name: String,
    pub supports_runtime_preview: bool,
    pub supports_shared_texture_viewports: bool,
}
