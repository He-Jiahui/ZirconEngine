#[path = "diagnostic_sections.rs"]
mod diagnostic_sections;
#[path = "full_chain_streams.rs"]
mod full_chain_streams;
#[path = "gpu_word_layout.rs"]
mod gpu_word_layout;
#[path = "launch_worklist_streams.rs"]
mod launch_worklist_streams;
#[path = "node_and_cluster_cull_streams.rs"]
mod node_and_cluster_cull_streams;
#[path = "render_path_streams.rs"]
mod render_path_streams;
#[path = "visbuffer64_streams.rs"]
mod visbuffer64_streams;

mod prelude {
    pub use zircon_runtime::core::framework::render::{
        RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryCullInputSnapshot,
        RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryDebugSnapshotDecodedStreams,
        RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError,
        RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint,
        RenderVirtualGeometryDebugSnapshotReadbackStreamReport,
        RenderVirtualGeometryDebugSnapshotReadbackStreamSection,
        RenderVirtualGeometryDebugSnapshotReadbackStreamSummary,
        RenderVirtualGeometryDebugSnapshotReadbackStreams, RenderVirtualGeometryDebugState,
        RenderVirtualGeometryExecutionState, RenderVirtualGeometryHardwareRasterizationRecord,
        RenderVirtualGeometryHardwareRasterizationSource,
        RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
        RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
        RenderVirtualGeometryNodeAndClusterCullDecodedStreams,
        RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
        RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
        RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
        RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
        RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
        RenderVirtualGeometryNodeAndClusterCullSource,
        RenderVirtualGeometryNodeAndClusterCullTraversalChildSource,
        RenderVirtualGeometryNodeAndClusterCullTraversalOp,
        RenderVirtualGeometryNodeAndClusterCullTraversalRecord,
        RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError,
        RenderVirtualGeometryNodeAndClusterCullWordStreams,
        RenderVirtualGeometryRenderPathDecodedStreams,
        RenderVirtualGeometryRenderPathWordStreamDecodeError,
        RenderVirtualGeometryRenderPathWordStreams, RenderVirtualGeometrySelectedCluster,
        RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64DecodedStream,
        RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64ReadbackStream,
        RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError,
        RenderVirtualGeometryVisBuffer64Source,
    };
}
