use zircon_framework::render::RenderFrameworkError;

use crate::GraphicsError;

pub(in crate::runtime::render_framework) fn render_framework_backend_error(
    error: GraphicsError,
) -> RenderFrameworkError {
    RenderFrameworkError::Backend(error.to_string())
}
