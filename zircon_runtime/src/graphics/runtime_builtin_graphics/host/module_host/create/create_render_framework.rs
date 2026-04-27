use std::sync::Arc;

use crate::core::framework::render::RenderFramework;
use crate::core::CoreHandle;
use crate::graphics::RenderFeatureDescriptor;
use crate::{GraphicsError, WgpuRenderFramework};

use super::resolve_project_asset_manager::resolve_project_asset_manager;

pub fn create_render_framework_with_render_features(
    core: &CoreHandle,
    render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
) -> Result<Arc<dyn RenderFramework>, GraphicsError> {
    let asset_manager = resolve_project_asset_manager(core)?;
    Ok(Arc::new(
        WgpuRenderFramework::new_with_plugin_render_features(asset_manager, render_features)?,
    ))
}
