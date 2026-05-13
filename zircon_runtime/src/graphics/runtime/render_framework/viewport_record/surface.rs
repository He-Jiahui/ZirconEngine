use crate::graphics::backend::ViewportSurface;

use super::viewport_record::ViewportRecord;

pub(in crate::graphics::runtime::render_framework) type ViewportSurfaceLease<'a> =
    SlotLease<'a, ViewportSurface>;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn bind_surface(
        &mut self,
        surface: ViewportSurface,
    ) {
        self.surface = Some(surface);
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
