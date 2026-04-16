//! Graphics module registration and manager services.

mod module_host;

pub use module_host::{
    create_render_server, create_render_service, create_render_service_with_icon_source,
    create_runtime_preview_renderer, create_shared_texture_render_service,
    create_shared_texture_render_service_with_icon_source, module_descriptor, WgpuDriver,
    WgpuRenderingManager, GRAPHICS_MODULE_NAME, RENDERING_MANAGER_NAME, RENDER_SERVER_NAME,
    WGPU_DRIVER_NAME,
};
