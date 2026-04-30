pub(super) use std::collections::{HashMap, HashSet};

pub(super) use super::super::virtual_geometry_node_and_cluster_cull_pass::{
    VirtualGeometryNodeAndClusterCullPassOutput, VirtualGeometryNodeAndClusterCullPassStoreParts,
};
pub(super) use super::seed_backed_execution_selection::{
    build_seed_backed_execution_selection_records, build_seed_backed_execution_selections,
    collect_execution_cluster_selection_collection_from_root_seeds, seed_backed_cluster_ordering,
    SeedBackedClusterOrdering, SeedBackedExecutionSelectionRecord,
};
pub(super) use super::selection_filter::collect_execution_cluster_selections_from_submission_keys;
pub(super) use crate::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryCullInputSnapshot, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExecutionState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryInstance, RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometryPage,
    RenderVirtualGeometrySelectedCluster,
};
pub(super) use crate::core::math::{Transform, Vec3};
pub(super) use crate::graphics::types::{
    VirtualGeometryClusterSelection, VirtualGeometryPrepareClusterState,
};

mod seed_backed_fallbacks;
mod seed_backed_ordering;
mod seed_backed_ranges;
mod selection_filter;
mod support;
