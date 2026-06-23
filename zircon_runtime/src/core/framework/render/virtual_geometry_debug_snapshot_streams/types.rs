use super::{
    RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
    RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBuffer64Source,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryNodeAndClusterCullWordStreams {
    pub source: RenderVirtualGeometryNodeAndClusterCullSource,
    pub global_state: Option<Vec<u32>>,
    pub dispatch_setup: Option<Vec<u32>>,
    pub launch_worklist: Option<Vec<u32>>,
    pub instance_seeds: Vec<u32>,
    pub instance_work_items: Vec<u32>,
    pub cluster_work_items: Vec<u32>,
    pub child_work_items: Vec<u32>,
    pub traversal_records: Vec<u32>,
    pub hierarchy_child_ids: Vec<u32>,
    pub page_request_ids: Vec<u32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryNodeAndClusterCullDecodedStreams {
    pub source: RenderVirtualGeometryNodeAndClusterCullSource,
    pub global_state: Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    pub dispatch_setup: Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>,
    pub launch_worklist: Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
    pub instance_seeds: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    pub instance_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>,
    pub cluster_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub child_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub traversal_records: Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub hierarchy_child_ids: Vec<u32>,
    pub page_request_ids: Vec<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryRenderPathWordStreams {
    pub selected_clusters_source: RenderVirtualGeometrySelectedClusterSource,
    pub hardware_rasterization_source: RenderVirtualGeometryHardwareRasterizationSource,
    pub selected_clusters: Vec<u32>,
    pub hardware_rasterization_records: Vec<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryRenderPathDecodedStreams {
    pub selected_clusters_source: RenderVirtualGeometrySelectedClusterSource,
    pub hardware_rasterization_source: RenderVirtualGeometryHardwareRasterizationSource,
    pub selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
    pub hardware_rasterization_records: Vec<RenderVirtualGeometryHardwareRasterizationRecord>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryVisBuffer64ReadbackStream {
    pub source: RenderVirtualGeometryVisBuffer64Source,
    pub clear_value: u64,
    pub entry_indices: Vec<u32>,
    pub packed_values: Vec<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryVisBuffer64DecodedStream {
    pub source: RenderVirtualGeometryVisBuffer64Source,
    pub clear_value: u64,
    pub entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryDebugSnapshotReadbackStreams {
    pub node_and_cluster_cull: RenderVirtualGeometryNodeAndClusterCullWordStreams,
    pub render_path: RenderVirtualGeometryRenderPathWordStreams,
    pub visbuffer64: RenderVirtualGeometryVisBuffer64ReadbackStream,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint {
    pub node_and_cluster_cull_u32_word_count: usize,
    pub render_path_u32_word_count: usize,
    pub visbuffer64_u32_word_count: usize,
    pub total_u32_word_count: usize,
    pub total_byte_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryDebugSnapshotReadbackStreamReport {
    pub footprint: RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint,
    pub summary: Option<RenderVirtualGeometryDebugSnapshotReadbackStreamSummary>,
    pub decode_error: Option<RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryDebugSnapshotDecodedStreams {
    pub node_and_cluster_cull: RenderVirtualGeometryNodeAndClusterCullDecodedStreams,
    pub render_path: RenderVirtualGeometryRenderPathDecodedStreams,
    pub visbuffer64: RenderVirtualGeometryVisBuffer64DecodedStream,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError {
    GlobalState { word_count: usize },
    DispatchSetup { word_count: usize },
    LaunchWorklist { word_count: usize },
    InstanceSeeds { word_count: usize },
    InstanceWorkItems { word_count: usize },
    ClusterWorkItems { word_count: usize },
    ChildWorkItems { word_count: usize },
    TraversalRecords { word_count: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderVirtualGeometryRenderPathWordStreamDecodeError {
    SelectedClusters { word_count: usize },
    HardwareRasterizationRecords { word_count: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError {
    MismatchedEntryAndValueCount {
        entry_index_count: usize,
        packed_value_count: usize,
    },
    InvalidPackedState {
        entry_index: u32,
        packed_value: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError {
    NodeAndClusterCull(RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError),
    RenderPath(RenderVirtualGeometryRenderPathWordStreamDecodeError),
    VisBuffer64(RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryDebugSnapshotReadbackStreamSummary {
    pub node_and_cluster_cull_source: RenderVirtualGeometryNodeAndClusterCullSource,
    pub node_and_cluster_cull_global_state_present: bool,
    pub node_and_cluster_cull_dispatch_setup_present: bool,
    pub node_and_cluster_cull_launch_worklist_present: bool,
    pub node_and_cluster_cull_instance_seed_count: usize,
    pub node_and_cluster_cull_instance_work_item_count: usize,
    pub node_and_cluster_cull_cluster_work_item_count: usize,
    pub node_and_cluster_cull_child_work_item_count: usize,
    pub node_and_cluster_cull_traversal_record_count: usize,
    pub node_and_cluster_cull_hierarchy_child_id_count: usize,
    pub node_and_cluster_cull_page_request_id_count: usize,
    pub selected_clusters_source: RenderVirtualGeometrySelectedClusterSource,
    pub selected_cluster_count: usize,
    pub hardware_rasterization_source: RenderVirtualGeometryHardwareRasterizationSource,
    pub hardware_rasterization_record_count: usize,
    pub visbuffer64_source: RenderVirtualGeometryVisBuffer64Source,
    pub visbuffer64_clear_value: u64,
    pub visbuffer64_entry_count: usize,
}
