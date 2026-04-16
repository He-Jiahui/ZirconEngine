//! ECS worlds, level systems, persistence, and render extraction.

pub type EntityId = u64;
pub type NodeId = EntityId;

mod components;
mod level_system;
mod module;
mod render_extract;
mod serializer;
mod world;

pub use components::{
    aspect_ratio_from_viewport_size, default_render_layer_mask, default_viewport_aspect_ratio,
    Active, ActiveInHierarchy, ActiveSelf, CameraComponent, DirectionalLight, DisplayMode,
    FallbackSkyboxKind, GridMode, GridOverlayExtract, HandleElementExtract, HandleOverlayExtract,
    Hierarchy, LocalTransform, MeshRenderer, Mobility, Name, NodeKind, NodeRecord, OverlayAxis,
    OverlayBillboardIcon, OverlayLineSegment, OverlayPickShape, OverlayWireShape,
    PreviewEnvironmentExtract, ProjectionMode, RenderBakedLightingExtract, RenderBloomSettings,
    RenderColorGradingSettings, RenderDirectionalLightSnapshot, RenderExtractPacket,
    RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion, RenderLayerMask,
    RenderMeshSnapshot, RenderOverlayExtract, RenderParticleSpriteSnapshot,
    RenderReflectionProbeSnapshot, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract, RenderVirtualGeometryPage,
    SceneGizmoBuildContext, SceneGizmoKind, SceneGizmoOverlayExtract, SceneGizmoProvider,
    SceneGizmoRegistry, SceneNode, SceneViewportExtractRequest, SceneViewportRenderPacket,
    SceneViewportSettings, SceneViewportTool, Schedule, SelectionAnchorExtract,
    SelectionHighlightExtract, SystemStage, TransformSpace, ViewOrientation,
    ViewportCameraSnapshot, ViewportIconId, WorldMatrix, WorldTransform,
};
pub use level_system::{LevelLifecycleState, LevelMetadata, LevelSystem};
pub use module::{
    create_default_level, load_level_asset, module_descriptor, DefaultLevelManager, WorldDriver,
    DEFAULT_LEVEL_MANAGER_NAME, LEVEL_MANAGER_NAME, SCENE_MODULE_NAME, WORLD_DRIVER_NAME,
};
pub use render_extract::{
    DebugOverlayExtract, GeometryExtract, LightingExtract, ParticleExtract, PostProcessExtract,
    RenderExtractContext, RenderExtractProducer, RenderFrameExtract, RenderViewExtract,
    RenderWorldSnapshotHandle, VisibilityInput, VisibilityRenderableInput,
};
pub use serializer::SceneAssetSerializer;
pub use world::{SceneProjectError, World};

pub type Scene = World;

#[cfg(test)]
mod tests;
