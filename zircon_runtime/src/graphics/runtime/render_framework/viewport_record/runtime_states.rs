use crate::graphics::{HybridGiRuntimeState, VirtualGeometryRuntimeState};

use super::{viewport_record::ViewportRecord, ViewportCameraHistoryKey};

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn ensure_hybrid_gi_runtime(
        &mut self,
        key: &ViewportCameraHistoryKey,
        provider: &dyn crate::graphics::HybridGiRuntimeProvider,
    ) -> &mut (dyn HybridGiRuntimeState + 'static) {
        if !self.hybrid_gi_runtimes.contains_key(key) {
            self.hybrid_gi_runtimes
                .insert(key.clone(), provider.create_state());
        }
        self.hybrid_gi_runtimes
            .get_mut(key)
            .expect("inserted Hybrid GI runtime")
            .as_mut()
    }

    pub(in crate::graphics::runtime::render_framework) fn clear_hybrid_gi_runtimes(&mut self) {
        self.hybrid_gi_runtimes.clear();
    }

    pub(in crate::graphics::runtime::render_framework) fn hybrid_gi_runtime_mut(
        &mut self,
        key: &ViewportCameraHistoryKey,
    ) -> Option<&mut (dyn HybridGiRuntimeState + 'static)> {
        self.hybrid_gi_runtimes.get_mut(key).map(Box::as_mut)
    }

    pub(in crate::graphics::runtime::render_framework) fn ensure_virtual_geometry_runtime(
        &mut self,
        key: &ViewportCameraHistoryKey,
        provider: &dyn crate::graphics::VirtualGeometryRuntimeProvider,
    ) -> &mut (dyn VirtualGeometryRuntimeState + 'static) {
        if !self.virtual_geometry_runtimes.contains_key(key) {
            self.virtual_geometry_runtimes
                .insert(key.clone(), provider.create_state());
        }
        self.virtual_geometry_runtimes
            .get_mut(key)
            .expect("inserted virtual geometry runtime")
            .as_mut()
    }

    pub(in crate::graphics::runtime::render_framework) fn clear_virtual_geometry_runtimes(
        &mut self,
    ) {
        self.virtual_geometry_runtimes.clear();
    }

    pub(in crate::graphics::runtime::render_framework) fn virtual_geometry_runtime_mut(
        &mut self,
        key: &ViewportCameraHistoryKey,
    ) -> Option<&mut (dyn VirtualGeometryRuntimeState + 'static)> {
        self.virtual_geometry_runtimes.get_mut(key).map(Box::as_mut)
    }
}

#[cfg(test)]
#[path = "tests/runtime_states.rs"]
mod tests;
