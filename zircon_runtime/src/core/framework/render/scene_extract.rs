//! 场景抽取的中立载荷由世界构建，再由图形模块映射为可见性、材质和 GPU 工作。
//! 这里记录场景身份与描述，不承担后端资源分配或命令提交。
mod hybrid_gi;
mod mesh;
mod particle;
mod post_process_settings;
mod snapshot;
mod virtual_geometry;

pub use hybrid_gi::{
    RenderHybridGiDebugView, RenderHybridGiExtract, RenderHybridGiFallbackReason,
    RenderHybridGiMode, RenderHybridGiProfile, RenderHybridGiQuality,
    RenderHybridGiResolvedSettings,
};
pub use mesh::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, RenderMeshLodSelection,
    RenderMeshSnapshot, RenderMeshStaticState, RENDER_MESH_STABLE_KEY_MAX_PRIMITIVE_ORDINAL,
    RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS,
};
pub use particle::{
    RenderParticleBillboardBasisSnapshot, RenderParticleBoundsSnapshot,
    RenderParticlePreviousSpriteSnapshot, RenderParticleSpriteIdentity,
    RenderParticleSpriteSnapshot,
};
pub use post_process_settings::{RenderBloomSettings, RenderColorGradingSettings};
pub use snapshot::{
    PreviewEnvironmentExtract, RenderExtractPacket, RenderSceneGeometryExtract,
    RenderSceneSnapshot, SceneViewportRenderPacket,
};
pub use virtual_geometry::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryHierarchyNode, RenderVirtualGeometryInstance, RenderVirtualGeometryPage,
    RenderVirtualGeometryPageDependency,
};
