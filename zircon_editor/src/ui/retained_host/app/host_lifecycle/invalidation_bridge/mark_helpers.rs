use crate::ui::retained_host::app::{HostInvalidationMask, RetainedEditorHost};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn mark_layout_dirty(&mut self) {
        self.invalidate_host(HostInvalidationMask::LAYOUT);
    }

    pub(in crate::ui::retained_host::app) fn mark_presentation_dirty(&mut self) {
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }

    pub(in crate::ui::retained_host::app) fn mark_render_and_presentation_dirty(&mut self) {
        self.invalidate_host(
            HostInvalidationMask::RENDER.union(HostInvalidationMask::PRESENTATION_DATA),
        );
    }
}
