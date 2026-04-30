use crate::graphics::runtime::{HybridGiRuntimeState, VirtualGeometryRuntimeState};
use crate::graphics::types::{
    HybridGiPrepareFrame, HybridGiResolveRuntime, HybridGiScenePrepareFrame,
    VirtualGeometryPrepareFrame,
};

pub(super) struct PreparedRuntimeSubmission {
    hybrid_gi_runtime: Option<HybridGiRuntimeState>,
    hybrid_gi_prepare: Option<HybridGiPrepareFrame>,
    hybrid_gi_scene_prepare: Option<HybridGiScenePrepareFrame>,
    hybrid_gi_resolve_runtime: Option<HybridGiResolveRuntime>,
    hybrid_gi_evictable_probe_ids: Vec<u32>,
    virtual_geometry_runtime: Option<VirtualGeometryRuntimeState>,
    virtual_geometry_prepare: Option<VirtualGeometryPrepareFrame>,
    virtual_geometry_evictable_page_ids: Vec<u32>,
}

impl PreparedRuntimeSubmission {
    pub(super) fn new(
        hybrid_gi_runtime: Option<HybridGiRuntimeState>,
        hybrid_gi_prepare: Option<HybridGiPrepareFrame>,
        hybrid_gi_scene_prepare: Option<HybridGiScenePrepareFrame>,
        hybrid_gi_resolve_runtime: Option<HybridGiResolveRuntime>,
        hybrid_gi_evictable_probe_ids: Vec<u32>,
        virtual_geometry_runtime: Option<VirtualGeometryRuntimeState>,
        virtual_geometry_prepare: Option<VirtualGeometryPrepareFrame>,
        virtual_geometry_evictable_page_ids: Vec<u32>,
    ) -> Self {
        Self {
            hybrid_gi_runtime,
            hybrid_gi_prepare,
            hybrid_gi_scene_prepare,
            hybrid_gi_resolve_runtime,
            hybrid_gi_evictable_probe_ids,
            virtual_geometry_runtime,
            virtual_geometry_prepare,
            virtual_geometry_evictable_page_ids,
        }
    }

    pub(super) fn hybrid_gi_runtime_mut(&mut self) -> Option<&mut HybridGiRuntimeState> {
        self.hybrid_gi_runtime.as_mut()
    }

    pub(super) fn hybrid_gi_prepare(&self) -> Option<&HybridGiPrepareFrame> {
        self.hybrid_gi_prepare.as_ref()
    }

    pub(super) fn hybrid_gi_scene_prepare(&self) -> Option<&HybridGiScenePrepareFrame> {
        self.hybrid_gi_scene_prepare.as_ref()
    }

    pub(super) fn hybrid_gi_resolve_runtime(&self) -> Option<&HybridGiResolveRuntime> {
        self.hybrid_gi_resolve_runtime.as_ref()
    }

    pub(super) fn take_hybrid_gi_evictable_probe_ids(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.hybrid_gi_evictable_probe_ids)
    }

    pub(super) fn virtual_geometry_runtime(&self) -> Option<&VirtualGeometryRuntimeState> {
        self.virtual_geometry_runtime.as_ref()
    }

    pub(super) fn virtual_geometry_runtime_mut(
        &mut self,
    ) -> Option<&mut VirtualGeometryRuntimeState> {
        self.virtual_geometry_runtime.as_mut()
    }

    pub(super) fn virtual_geometry_prepare(&self) -> Option<&VirtualGeometryPrepareFrame> {
        self.virtual_geometry_prepare.as_ref()
    }

    pub(super) fn take_virtual_geometry_evictable_page_ids(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.virtual_geometry_evictable_page_ids)
    }

    pub(super) fn into_runtime_states(
        self,
    ) -> (
        Option<HybridGiRuntimeState>,
        Option<VirtualGeometryRuntimeState>,
    ) {
        (self.hybrid_gi_runtime, self.virtual_geometry_runtime)
    }
}
