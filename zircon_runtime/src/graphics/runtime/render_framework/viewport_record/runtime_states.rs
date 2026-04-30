use crate::runtime::{HybridGiRuntimeState, VirtualGeometryRuntimeState};

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn hybrid_gi_runtime(
        &self,
    ) -> Option<&HybridGiRuntimeState> {
        self.hybrid_gi_runtime.as_ref()
    }

    pub(in crate::graphics::runtime::render_framework) fn virtual_geometry_runtime(
        &self,
    ) -> Option<&VirtualGeometryRuntimeState> {
        self.virtual_geometry_runtime.as_ref()
    }

    pub(in crate::graphics::runtime::render_framework) fn replace_runtime_states(
        &mut self,
        hybrid_gi_runtime: Option<HybridGiRuntimeState>,
        virtual_geometry_runtime: Option<VirtualGeometryRuntimeState>,
    ) {
        self.hybrid_gi_runtime = hybrid_gi_runtime;
        self.virtual_geometry_runtime = virtual_geometry_runtime;
    }
}
