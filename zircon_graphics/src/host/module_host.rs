use crate::{GraphicsError, RenderService, RuntimePreviewRenderer, SharedTextureRenderService};
use std::sync::Arc;
use winit::window::Window;
use zircon_asset::{ProjectAssetManager, PROJECT_ASSET_MANAGER_NAME};
use zircon_core::{
    CoreHandle, DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject,
    StartupMode,
};
use zircon_manager::{
    RenderingBackendInfo, RenderingManager as RenderingManagerFacade, RenderingManagerHandle,
};
use zircon_module::{factory, qualified_name};

pub const GRAPHICS_MODULE_NAME: &str = "GraphicsModule";
pub const WGPU_DRIVER_NAME: &str = "GraphicsModule.Driver.WgpuDriver";
pub const RENDERING_MANAGER_NAME: &str = zircon_manager::RENDERING_MANAGER_NAME;

#[derive(Debug, Default)]
pub struct WgpuDriver;

#[derive(Debug, Default)]
pub struct WgpuRenderingManager;

impl WgpuRenderingManager {
    pub fn spawn_render_service(
        &self,
        asset_manager: Arc<ProjectAssetManager>,
    ) -> Result<RenderService, GraphicsError> {
        RenderService::spawn(asset_manager)
    }

    pub fn spawn_shared_texture_render_service(
        &self,
        device: wgpu::Device,
        queue: wgpu::Queue,
        asset_manager: Arc<ProjectAssetManager>,
    ) -> Result<SharedTextureRenderService, GraphicsError> {
        SharedTextureRenderService::spawn_with_device(device, queue, asset_manager)
    }

    pub fn create_runtime_preview_renderer(
        &self,
        window: Arc<Window>,
        asset_manager: Arc<ProjectAssetManager>,
    ) -> Result<RuntimePreviewRenderer, GraphicsError> {
        RuntimePreviewRenderer::new(window, asset_manager)
    }
}

pub fn create_render_service(core: &CoreHandle) -> Result<RenderService, GraphicsError> {
    let asset_manager = core
        .resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)
        .map_err(|error| GraphicsError::Asset(error.to_string()))?;
    RenderService::spawn(asset_manager)
}

pub fn create_shared_texture_render_service(
    core: &CoreHandle,
    device: wgpu::Device,
    queue: wgpu::Queue,
) -> Result<SharedTextureRenderService, GraphicsError> {
    let asset_manager = core
        .resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)
        .map_err(|error| GraphicsError::Asset(error.to_string()))?;
    SharedTextureRenderService::spawn_with_device(device, queue, asset_manager)
}

pub fn create_runtime_preview_renderer(
    core: &CoreHandle,
    window: Arc<Window>,
) -> Result<RuntimePreviewRenderer, GraphicsError> {
    let asset_manager = core
        .resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)
        .map_err(|error| GraphicsError::Asset(error.to_string()))?;
    RuntimePreviewRenderer::new(window, asset_manager)
}

impl RenderingManagerFacade for WgpuRenderingManager {
    fn backend_info(&self) -> RenderingBackendInfo {
        RenderingBackendInfo {
            backend_name: "wgpu".to_string(),
            supports_runtime_preview: true,
            supports_shared_texture_viewports: true,
        }
    }
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        GRAPHICS_MODULE_NAME,
        "Rendering device abstraction and scene rendering",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(GRAPHICS_MODULE_NAME, ServiceKind::Driver, "WgpuDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(WgpuDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(GRAPHICS_MODULE_NAME, ServiceKind::Manager, "RenderingManager"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| {
            let manager = Arc::new(WgpuRenderingManager);
            Ok(Arc::new(RenderingManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
}
