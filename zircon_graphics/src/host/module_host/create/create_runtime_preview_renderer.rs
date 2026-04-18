use std::sync::Arc;

use winit::window::Window;
use zircon_core::CoreHandle;

use crate::{GraphicsError, RuntimePreviewRenderer};

use super::resolve_project_asset_manager::resolve_project_asset_manager;

pub fn create_runtime_preview_renderer(
    core: &CoreHandle,
    window: Arc<dyn Window>,
) -> Result<RuntimePreviewRenderer, GraphicsError> {
    let asset_manager = resolve_project_asset_manager(core)?;
    RuntimePreviewRenderer::new(window, asset_manager)
}
