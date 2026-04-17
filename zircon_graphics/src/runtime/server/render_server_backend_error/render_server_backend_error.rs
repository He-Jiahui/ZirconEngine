use zircon_render_server::RenderServerError;

use crate::GraphicsError;

pub(in crate::runtime::server) fn render_server_backend_error(
    error: GraphicsError,
) -> RenderServerError {
    RenderServerError::Backend(error.to_string())
}
