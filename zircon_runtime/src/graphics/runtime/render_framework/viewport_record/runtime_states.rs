use crate::{HybridGiRuntimeState, VirtualGeometryRuntimeState};

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn ensure_hybrid_gi_runtime(
        &mut self,
        provider: &dyn crate::HybridGiRuntimeProvider,
    ) -> &mut (dyn HybridGiRuntimeState + 'static) {
        if self.hybrid_gi_runtime.is_none() {
            self.hybrid_gi_runtime = Some(provider.create_state());
        }
        self.hybrid_gi_runtime
            .as_deref_mut()
            .expect("hybrid GI runtime inserted above")
    }

    pub(in crate::graphics::runtime::render_framework) fn clear_hybrid_gi_runtime(&mut self) {
        self.hybrid_gi_runtime = None;
    }

    pub(in crate::graphics::runtime::render_framework) fn hybrid_gi_runtime_mut(
        &mut self,
    ) -> Option<&mut (dyn HybridGiRuntimeState + 'static)> {
        self.hybrid_gi_runtime.as_deref_mut()
    }

    pub(in crate::graphics::runtime::render_framework) fn ensure_virtual_geometry_runtime(
        &mut self,
        provider: &dyn crate::VirtualGeometryRuntimeProvider,
    ) -> &mut (dyn VirtualGeometryRuntimeState + 'static) {
        if self.virtual_geometry_runtime.is_none() {
            self.virtual_geometry_runtime = Some(provider.create_state());
        }
        self.virtual_geometry_runtime
            .as_deref_mut()
            .expect("virtual geometry runtime inserted above")
    }

    pub(in crate::graphics::runtime::render_framework) fn clear_virtual_geometry_runtime(
        &mut self,
    ) {
        self.virtual_geometry_runtime = None;
    }

    pub(in crate::graphics::runtime::render_framework) fn virtual_geometry_runtime_mut(
        &mut self,
    ) -> Option<&mut (dyn VirtualGeometryRuntimeState + 'static)> {
        self.virtual_geometry_runtime.as_deref_mut()
    }
}
