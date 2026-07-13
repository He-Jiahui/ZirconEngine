use std::sync::{Mutex, MutexGuard};

use crate::core::framework::render::PlanarReflectionUpdateState;
#[cfg(test)]
use crate::core::framework::render::{RenderCapabilitySummary, RenderFrameworkError};
use crate::core::TaskPool;
#[cfg(test)]
use crate::core::{math::UVec2, resource::ResourceId};
#[cfg(test)]
use crate::graphics::shader::ShaderVariantCacheDisk;

#[cfg(test)]
use super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::render_framework_state::RenderFrameworkState;

pub struct WgpuRenderFramework {
    pub(in crate::graphics::runtime::render_framework) state: Mutex<RenderFrameworkState>,
    pub(in crate::graphics::runtime::render_framework) operation_lock: Mutex<()>,
    pub(in crate::graphics::runtime::render_framework) compute_task_pool: TaskPool,
    pub(in crate::graphics::runtime::render_framework) planar_reflection_updates:
        Mutex<PlanarReflectionUpdateState>,
}

impl WgpuRenderFramework {
    pub(in crate::graphics::runtime::render_framework) fn lock_operation(
        &self,
    ) -> MutexGuard<'_, ()> {
        #[cfg(feature = "profiling")]
        crate::profile_scope!("runtime", "render_framework.wait", "operation_lock");
        self.operation_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::graphics::runtime::render_framework) fn lock_state(
        &self,
    ) -> MutexGuard<'_, RenderFrameworkState> {
        #[cfg(feature = "profiling")]
        crate::profile_scope!("runtime", "render_framework.wait", "state");
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::graphics::runtime::render_framework) fn lock_planar_reflection_updates(
        &self,
    ) -> MutexGuard<'_, PlanarReflectionUpdateState> {
        match self.planar_reflection_updates.lock() {
            Ok(updates) => updates,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// Invalidates one on-demand planar probe so the next camera loop submits
    /// its mirror capture before the owning main camera.
    pub fn request_planar_reflection_capture(&self, probe_id: u64) {
        self.lock_planar_reflection_updates().mark_dirty(probe_id);
    }

    #[cfg(test)]
    pub(crate) fn override_capabilities_for_tests(&self, capabilities: RenderCapabilitySummary) {
        self.lock_state().stats.capabilities = capabilities;
    }

    #[cfg(test)]
    pub(crate) fn request_next_created_viewport_graphics_debugger_capture_for_tests(&self) {
        self.lock_state()
            .graphics_debugger
            .request_next_created_viewport_capture();
    }

    #[cfg(test)]
    pub(crate) fn replace_shader_variant_disk_cache_for_tests(
        &self,
        cache: ShaderVariantCacheDisk,
    ) {
        self.lock_state()
            .renderer
            .replace_shader_variant_disk_cache_for_tests(cache);
    }

    #[cfg(test)]
    pub(crate) fn read_output_target_texture_rgba_for_tests(
        &self,
        texture_id: ResourceId,
    ) -> Result<Option<(UVec2, Vec<u8>)>, RenderFrameworkError> {
        let _operation_guard = self.lock_operation();
        let state = self.lock_state();
        state
            .renderer
            .read_output_target_texture_rgba_for_tests(&texture_id)
            .map_err(render_framework_backend_error)
    }

    #[cfg(test)]
    pub(crate) fn last_scene_velocity_readback_rg16_float_bytes_for_tests(
        &self,
    ) -> Option<Vec<u8>> {
        let _operation_guard = self.lock_operation();
        self.lock_state()
            .renderer
            .last_scene_velocity_readback_rg16_float_bytes()
    }

    #[cfg(test)]
    pub(crate) fn reflection_probe_upload_diagnostics_for_tests(
        &self,
    ) -> (usize, usize, usize, usize, Option<String>) {
        let _operation_guard = self.lock_operation();
        self.lock_state()
            .renderer
            .reflection_probe_upload_diagnostics_for_tests()
    }

    #[cfg(test)]
    pub(crate) fn reflection_probe_gpu_upload_diagnostics_for_tests(
        &self,
    ) -> Result<(u32, [[f32; 4]; 2], [[u16; 4]; 2]), RenderFrameworkError> {
        let _operation_guard = self.lock_operation();
        self.lock_state()
            .renderer
            .reflection_probe_gpu_upload_diagnostics_for_tests()
            .map_err(render_framework_backend_error)
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::Arc;

    use crate::asset::pipeline::manager::ProjectAssetManager;

    use super::*;

    #[test]
    fn wgpu_render_framework_accessors_recover_poisoned_locks() {
        let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default()))
            .expect("framework should initialize for lock recovery test");

        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = framework.operation_lock.lock().unwrap();
            panic!("poison operation lock");
        }));
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = framework.state.lock().unwrap();
            panic!("poison render framework state lock");
        }));

        drop(framework.lock_operation());
        assert!(framework.lock_state().viewports.is_empty());
    }
}
