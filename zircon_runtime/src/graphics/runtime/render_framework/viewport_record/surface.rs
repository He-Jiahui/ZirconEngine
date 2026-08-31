use crate::core::math::UVec2;
use crate::graphics::{backend::ViewportSurface, runtime::ViewportFrameHistory};

use super::viewport_record::ViewportRecord;

pub(in crate::graphics::runtime::render_framework) type ViewportSurfaceLease<'a> =
    SlotLease<'a, ViewportSurface>;

impl ViewportRecord {
    /// Publishes a prepared surface and its extent while the render-framework
    /// state lock is held. The caller must fence submissions before invoking
    /// this so the previous surface and resolution-dependent histories are no
    /// longer in use.
    pub(in crate::graphics::runtime::render_framework) fn replace_surface_and_extent(
        &mut self,
        surface: ViewportSurface,
        size: UVec2,
    ) -> Vec<ViewportFrameHistory> {
        self.descriptor.size = size;
        let previous_surface = self.surface.replace(surface);
        let histories = std::mem::take(&mut self.camera_histories)
            .into_values()
            .collect();
        self.temporal_frame_index = 0;
        self.last_capture_pipeline = None;
        self.pending_capture_profiles.clear();
        self.last_promoted_capture_generation = None;
        self.hybrid_gi_runtimes.clear();
        self.virtual_geometry_runtimes.clear();
        self.light_grid_reports.clear();
        self.virtual_geometry_debug_snapshots.clear();
        self.last_capture = None;
        self.last_visible_spatial_query = None;
        self.motion_vector_cameras.clear();
        self.particle_previous_sprites.clear();
        self.capture_mailbox = Default::default();
        drop(previous_surface);
        histories
    }

    pub(in crate::graphics::runtime::render_framework) fn unbind_surface(&mut self) {
        self.surface = None;
    }

    pub(in crate::graphics::runtime::render_framework) fn has_surface(&self) -> bool {
        self.surface.is_some()
    }

    pub(in crate::graphics::runtime::render_framework) fn lease_surface(
        &mut self,
    ) -> Option<ViewportSurfaceLease<'_>> {
        SlotLease::take(&mut self.surface)
    }
}

pub(in crate::graphics::runtime::render_framework) struct SlotLease<'a, T> {
    slot: &'a mut Option<T>,
    value: Option<T>,
}

impl<'a, T> SlotLease<'a, T> {
    fn take(slot: &'a mut Option<T>) -> Option<Self> {
        let value = slot.take()?;
        Some(Self {
            slot,
            value: Some(value),
        })
    }

    pub(in crate::graphics::runtime::render_framework) fn value_mut(&mut self) -> &mut T {
        self.value.as_mut().expect("leased slot value")
    }

    pub(in crate::graphics::runtime::render_framework) fn restore(mut self) {
        self.restore_inner();
    }

    fn restore_inner(&mut self) {
        if self.slot.is_none() {
            *self.slot = self.value.take();
        }
    }
}

impl<T> Drop for SlotLease<'_, T> {
    fn drop(&mut self) {
        self.restore_inner();
    }
}

#[cfg(test)]
mod tests {
    use super::SlotLease;

    #[test]
    fn graphics_surface_slot_lease_restores_value_on_drop() {
        let mut slot = Some(7);
        {
            let mut lease = SlotLease::take(&mut slot).expect("slot has value");
            *lease.value_mut() = 11;
        }

        assert_eq!(slot, Some(11));
    }

    #[test]
    fn graphics_surface_slot_lease_restores_value_on_explicit_restore() {
        let mut slot = Some(3);
        let mut lease = SlotLease::take(&mut slot).expect("slot has value");
        *lease.value_mut() = 5;

        lease.restore();

        assert_eq!(slot, Some(5));
    }
}
