#[path = "context/mod.rs"]
mod context;
#[path = "culling/mod.rs"]
mod culling;
#[path = "declarations/mod.rs"]
mod declarations;
#[path = "occlusion/mod.rs"]
mod occlusion;
#[path = "planning/mod.rs"]
mod planning;
#[path = "spatial_query.rs"]
mod spatial_query;
#[path = "static_index/mod.rs"]
mod static_index;
#[path = "view_context/mod.rs"]
mod view_context;

pub use declarations::{
    VisibilityBatch, VisibilityBatchKey, VisibilityBounds, VisibilityBvhInstance,
    VisibilityBvhUpdatePlan, VisibilityBvhUpdateStrategy, VisibilityContext, VisibilityDrawCommand,
    VisibilityHistoryEntry, VisibilityHistorySnapshot, VisibilityHybridGiFeedback,
    VisibilityHybridGiProbe, VisibilityHybridGiUpdatePlan, VisibilityInstanceUploadPlan,
    VisibilityParticleUploadPlan, VisibilityVirtualGeometryCluster,
    VisibilityVirtualGeometryDrawSegment, VisibilityVirtualGeometryFeedback,
    VisibilityVirtualGeometryPageUploadPlan,
};
pub use occlusion::{
    HzbBuildPlan, HzbBuilder, HzbOcclusionCullReadbackStats, HzbOcclusionCullReport,
    HzbOcclusionIndirectArgsReadbackSummary,
};
pub(crate) use spatial_query::VisibleSpatialQuery;
pub(crate) use static_index::VisibilityStaticIndex;
pub use static_index::VisibilityStaticIndexReport;
pub use view_context::{
    FrameVisibility, ViewCullingStats, ViewVisibilityContext, VisibilityViewKey,
};

pub(crate) use planning::cluster_ids_for_stable_instance_key;
