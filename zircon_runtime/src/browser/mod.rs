//! Single-threaded browser composition over the existing engine modules.
use crate::asset::runtime::browser::{BrowserAssetLimits, BrowserAssetSource};
use crate::asset::{runtime_asset_manager_handle, RuntimeAssetManagerAccess};
use crate::core::framework::input::{InputEvent, InputManager};
use crate::core::framework::render::{
    RenderExtractContext, RenderExtractProducer, RenderFrameTiming, RenderFramework,
    RenderSubmissionConfig, RenderViewportDescriptor, RenderViewportHandle,
    RenderViewportSurfaceDescriptor, RenderWorldSnapshotHandle, SceneViewportExtractRequest,
};
use crate::core::manager::{
    input_manager_handle, render_framework_handle, resolve_manager_service,
    RegisteredManagerService, RENDER_FRAMEWORK_NAME,
};
use crate::core::math::UVec2;
use crate::core::runtime::{
    modules::{FrameCountModule, TasksModule, TimeModule},
    ServiceObject,
};
use crate::core::{
    CoreError, CoreRuntime, InitLevel, ManagerDescriptor, ModuleDescriptor, ServiceKind,
    StartupMode,
};
use crate::engine_module::{factory, qualified_name, EngineModule};
use crate::graphics::WgpuRenderFramework;
use crate::scene::{LevelMetadata, LevelSystem, World, WorldRuntimeExtensionPlan};
use serde::Serialize;
use std::fmt::Debug;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use zr_rhi::RenderNativeSurfaceTarget;

mod close;
mod frame;
mod input;
mod open;
mod turn;
mod ui;
pub use input::BrowserInput;
#[cfg(feature = "memory-validation")]
mod allocator;
#[cfg(feature = "memory-validation")]
pub use allocator::{browser_heap_snapshot, BrowserHeapSnapshot};

static LIVE_ENGINES: AtomicU64 = AtomicU64::new(0);
pub fn registered_browser_canvases() -> usize {
    zr_rhi_wgpu::registered_canvas_surface_count()
}
pub fn live_browser_engines() -> u64 {
    LIVE_ENGINES.load(Ordering::Acquire)
}
fn error(error: impl Debug) -> String {
    format!("{error:?}")
}

#[derive(Debug, Serialize)]
pub struct BrowserFrameReceipt {
    pub frame: u64,
    pub presented: bool,
    pub animation_poses: usize,
    pub ui_commands: usize,
    pub pending_assets: usize,
    pub omitted_input_events: u64,
    pub submitted_frames: u64,
    pub visible_instances: usize,
    /// Engine texture residency and graph peaks, excluding browser/driver allocations.
    pub texture_resident_bytes: u64,
    pub transient_texture_peak_bytes: u64,
    pub transient_buffer_peak_bytes: u64,
    pub staging_bytes: u64,
}
#[derive(Debug, Serialize)]
pub struct BrowserCloseReceipt {
    pub modules_closed: bool,
    pub task_graph_joined: bool,
    pub pending_assets: usize,
}
/// The JS owner must await frame/close; a frame never waits on a browser thread.
pub struct BrowserEngine {
    core: Option<CoreRuntime>,
    source: Arc<BrowserAssetSource>,
    renderer: Option<Arc<WgpuRenderFramework>>,
    level: Option<LevelSystem>,
    ui: Option<ui::BrowserUi>,
    viewport: Option<RenderViewportHandle>,
    canvas: web_sys::HtmlCanvasElement,
    identity: u64,
    input: Arc<BrowserInput>,
    frame_events: Vec<InputEvent>,
    last_timestamp: Option<f64>,
    lifecycle_generation: u64,
    frame: u64,
    size: UVec2,
    closing: bool,
}
impl BrowserEngine {
    pub fn input(&self) -> Arc<BrowserInput> {
        Arc::clone(&self.input)
    }
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        if self.closing {
            return Err("browser engine is closing".into());
        }
        if width > 4096 || height > 4096 {
            return Err("browser viewport exceeds the 4096 pixel dimension limit".into());
        }
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        // The next serial frame reconfigures the surface after prior validation.
        Ok(())
    }
    fn bind_surface(&self, size: UVec2) -> Result<(), String> {
        let renderer = self
            .renderer
            .as_ref()
            .ok_or_else(|| "renderer is closed".to_string())?;
        renderer
            .bind_viewport_surface(
                self.viewport
                    .ok_or_else(|| "viewport is closed".to_string())?,
                RenderViewportSurfaceDescriptor {
                    size,
                    target: RenderNativeSurfaceTarget::Canvas {
                        identity: self.identity,
                    },
                },
            )
            .map_err(error)
    }
}
fn platform_module(input: Arc<BrowserInput>) -> ModuleDescriptor {
    const NAME: &str = crate::core::framework::platform::PLATFORM_MODULE_NAME;
    ModuleDescriptor::new(
        NAME,
        "Browser visibility, lifecycle and bounded input ingress",
    )
    .with_init_level(InitLevel::Services)
    .with_manager(ManagerDescriptor::new(
        qualified_name(NAME, ServiceKind::Manager, "BrowserInput"),
        StartupMode::Immediate,
        Vec::new(),
        factory(move |_| Ok(Arc::clone(&input) as ServiceObject)),
    ))
}
