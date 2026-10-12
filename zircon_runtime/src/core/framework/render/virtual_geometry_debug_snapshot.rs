//! 虚拟几何调试快照汇聚提取输入、CPU 对照、页依赖、驻留载荷和执行来源；它只观测提交状态，不接管 provider 的页生命周期。
#[path = "debug/bvh_visualization.rs"]
mod bvh_visualization;
#[path = "debug/cpu_reference.rs"]
mod cpu_reference;
#[path = "debug/cull_input.rs"]
mod cull_input;
#[path = "debug/encoding.rs"]
mod encoding;
#[path = "debug/execution.rs"]
mod execution;
#[path = "debug/node_and_cluster_cull.rs"]
mod node_and_cluster_cull;
#[path = "debug/page_payload.rs"]
mod page_payload;
#[path = "debug/snapshot.rs"]
mod snapshot;
#[path = "debug/sources.rs"]
mod sources;

pub use bvh_visualization::{
    RenderVirtualGeometryBvhVisualizationInstance, RenderVirtualGeometryBvhVisualizationNode,
};
pub use cpu_reference::{
    RenderVirtualGeometryCpuReferenceDepthClusterMapEntry,
    RenderVirtualGeometryCpuReferenceInstance, RenderVirtualGeometryCpuReferenceLeafCluster,
    RenderVirtualGeometryCpuReferenceMipClusterMapEntry,
    RenderVirtualGeometryCpuReferenceNodeVisit,
    RenderVirtualGeometryCpuReferencePageClusterMapEntry,
    RenderVirtualGeometryCpuReferencePageDependencyEntry,
    RenderVirtualGeometryCpuReferenceSelectedCluster,
};
pub use cull_input::RenderVirtualGeometryCullInputSnapshot;
pub use execution::{
    RenderVirtualGeometryExecutionSegment, RenderVirtualGeometryExecutionState,
    RenderVirtualGeometryHardwareRasterizationRecord, RenderVirtualGeometryPageRequestInspection,
    RenderVirtualGeometryResidentPageInspection, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySubmissionEntry, RenderVirtualGeometrySubmissionRecord,
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBufferMark,
};
pub use node_and_cluster_cull::{
    RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
    RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullTraversalChildSource,
    RenderVirtualGeometryNodeAndClusterCullTraversalOp,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord,
};
pub use page_payload::{
    RenderVirtualGeometryPagePayload, RenderVirtualGeometryPagePayloadClusterRange,
    RenderVirtualGeometryPagePayloadVertex,
};
pub use snapshot::RenderVirtualGeometryDebugSnapshot;
pub use sources::{
    RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source,
};

#[cfg(test)]
#[path = "debug/tests/cases.rs"]
mod tests;
