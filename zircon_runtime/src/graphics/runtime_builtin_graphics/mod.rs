//! Graphics module-host registration absorbed into the runtime layer.

mod host;

use crate::engine_module::{EngineModule, ModuleDescriptor};
use crate::graphics::RenderFeatureDescriptor;

pub use host::{
    module_descriptor, module_descriptor_with_render_features, GRAPHICS_MODULE_NAME,
    RENDERING_MANAGER_NAME, RENDER_FRAMEWORK_NAME,
};

#[derive(Clone, Debug, Default)]
pub struct GraphicsModule {
    render_features: Vec<RenderFeatureDescriptor>,
}

impl GraphicsModule {
    pub fn with_render_features(
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    ) -> Self {
        Self {
            render_features: render_features.into_iter().collect(),
        }
    }

    pub fn render_features(&self) -> &[RenderFeatureDescriptor] {
        &self.render_features
    }
}

impl EngineModule for GraphicsModule {
    fn module_name(&self) -> &'static str {
        GRAPHICS_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Rendering device abstraction and scene rendering"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor_with_render_features(self.render_features.clone())
    }
}
