use thiserror::Error;
use zircon_math::UVec2;
use zircon_resource::ResourceId;
use zircon_scene::{EntityId, RenderFrameExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle};

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
    #[error("asset loading failed: {0}")]
    Asset(String),
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
pub struct ViewportFrameTextureHandle {
    pub width: u32,
    pub height: u32,
    pub texture: wgpu::Texture,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsages,
    pub generation: u64,
}

#[derive(Clone, Debug)]
pub struct EditorOrRuntimeFrame {
    pub scene: RenderSceneSnapshot,
    pub extract: RenderFrameExtract,
    pub viewport: ViewportState,
    pub(crate) hybrid_gi_prepare: Option<HybridGiPrepareFrame>,
    pub(crate) virtual_geometry_prepare: Option<VirtualGeometryPrepareFrame>,
}

impl EditorOrRuntimeFrame {
    pub fn from_snapshot(scene: RenderSceneSnapshot, viewport: ViewportState) -> Self {
        let extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(0), scene.clone());
        Self {
            scene,
            extract,
            viewport,
            hybrid_gi_prepare: None,
            virtual_geometry_prepare: None,
        }
    }

    pub fn from_extract(extract: RenderFrameExtract, viewport: ViewportState) -> Self {
        let scene = extract.to_legacy_snapshot();
        Self {
            scene,
            extract,
            viewport,
            hybrid_gi_prepare: None,
            virtual_geometry_prepare: None,
        }
    }

    pub(crate) fn with_hybrid_gi_prepare(mut self, prepare: Option<HybridGiPrepareFrame>) -> Self {
        self.hybrid_gi_prepare = prepare;
        self
    }

    pub(crate) fn with_virtual_geometry_prepare(
        mut self,
        prepare: Option<VirtualGeometryPrepareFrame>,
    ) -> Self {
        self.virtual_geometry_prepare = prepare;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HybridGiPrepareProbe {
    pub(crate) probe_id: u32,
    pub(crate) slot: u32,
    pub(crate) ray_budget: u32,
    pub(crate) irradiance_rgb: [u8; 3],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HybridGiPrepareUpdateRequest {
    pub(crate) probe_id: u32,
    pub(crate) ray_budget: u32,
    pub(crate) generation: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiPrepareFrame {
    pub(crate) resident_probes: Vec<HybridGiPrepareProbe>,
    pub(crate) pending_updates: Vec<HybridGiPrepareUpdateRequest>,
    pub(crate) scheduled_trace_region_ids: Vec<u32>,
    pub(crate) evictable_probe_ids: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VirtualGeometryPrepareClusterState {
    Resident,
    PendingUpload,
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryPrepareCluster {
    pub(crate) entity: EntityId,
    pub(crate) cluster_id: u32,
    pub(crate) page_id: u32,
    pub(crate) lod_level: u8,
    pub(crate) resident_slot: Option<u32>,
    pub(crate) state: VirtualGeometryPrepareClusterState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryPreparePage {
    pub(crate) page_id: u32,
    pub(crate) slot: u32,
    pub(crate) size_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryPrepareRequest {
    pub(crate) page_id: u32,
    pub(crate) size_bytes: u64,
    pub(crate) generation: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryPrepareFrame {
    pub(crate) visible_entities: Vec<EntityId>,
    pub(crate) visible_clusters: Vec<VirtualGeometryPrepareCluster>,
    pub(crate) resident_pages: Vec<VirtualGeometryPreparePage>,
    pub(crate) pending_page_requests: Vec<VirtualGeometryPrepareRequest>,
    pub(crate) evictable_pages: Vec<VirtualGeometryPreparePage>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum GpuResourceHandle {
    Texture(ResourceId),
    Model(ResourceId),
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
