use crate::graphics::runtime::{HybridGiRuntimeState, VirtualGeometryRuntimeState};
use crate::graphics::types::{HybridGiPrepareFrame, HybridGiResolveRuntime, VirtualGeometryPrepareFrame};

pub(super) struct PreparedRuntimeSubmission {
    pub(super) hybrid_gi_runtime: Option<HybridGiRuntimeState>,
    pub(super) hybrid_gi_prepare: Option<HybridGiPrepareFrame>,
    pub(super) hybrid_gi_resolve_runtime: Option<HybridGiResolveRuntime>,
    pub(super) hybrid_gi_evictable_probe_ids: Vec<u32>,
    pub(super) virtual_geometry_runtime: Option<VirtualGeometryRuntimeState>,
    pub(super) virtual_geometry_prepare: Option<VirtualGeometryPrepareFrame>,
    pub(super) virtual_geometry_evictable_page_ids: Vec<u32>,
}
