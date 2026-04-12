use thiserror::Error;
use zircon_asset::{MeshSource, TextureSource};
use zircon_math::UVec2;
use zircon_scene::RenderSceneSnapshot;

#[derive(Debug, Error)]
pub enum GraphicsError {
    #[error("wgpu surface error: {0}")]
    Surface(#[from] wgpu::SurfaceError),
    #[error("surface creation failed: {0}")]
    SurfaceCreation(#[from] wgpu::CreateSurfaceError),
    #[error("no compatible adapter found")]
    NoAdapter,
    #[error("request device failed: {0}")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
    #[error("asset channel failure: {0}")]
    Channel(String),
    #[error("thread bootstrap failure: {0}")]
    ThreadBootstrap(String),
    #[error("buffer map failed: {0}")]
    BufferMap(String),
}

#[derive(Clone, Debug)]
pub struct ViewportState {
    pub size: UVec2,
}

impl ViewportState {
    pub fn new(size: UVec2) -> Self {
        Self {
            size: UVec2::new(size.x.max(1), size.y.max(1)),
        }
    }
}

impl Default for ViewportState {
    fn default() -> Self {
        Self::new(UVec2::new(960, 540))
    }
}

#[derive(Clone, Debug)]
pub struct ViewportFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub generation: u64,
}

#[derive(Clone, Debug)]
pub struct EditorOrRuntimeFrame {
    pub scene: RenderSceneSnapshot,
    pub viewport: ViewportState,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum GpuResourceHandle {
    Texture(TextureSource),
    Mesh(MeshSource),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Debug)]
pub enum ViewportInput {
    PointerMoved(zircon_math::Vec2),
    LeftPressed(zircon_math::Vec2),
    LeftReleased,
    RightPressed(zircon_math::Vec2),
    RightReleased,
    MiddlePressed(zircon_math::Vec2),
    MiddleReleased,
    Scrolled(f32),
    Resized(UVec2),
}

#[derive(Clone, Debug, Default)]
pub struct ViewportFeedback {
    pub hovered_axis: Option<GizmoAxis>,
    pub transformed_node: Option<zircon_scene::NodeId>,
    pub camera_updated: bool,
}
