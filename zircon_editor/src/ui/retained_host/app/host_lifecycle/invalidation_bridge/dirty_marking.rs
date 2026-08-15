use crate::ui::retained_host::app::{HostInvalidationMask, RetainedEditorHost};
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn mark_layout_dirty(&mut self) {
        self.invalidate_host(HostInvalidationMask::LAYOUT);
    }

    pub(in crate::ui::retained_host::app) fn mark_presentation_dirty(&mut self) {
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }

    pub(in crate::ui::retained_host::app) fn mark_presentation_dirty_for_view(
        &mut self,
        view: &ViewInstanceId,
    ) {
        self.invalidate_host_for_view(view, HostInvalidationMask::PRESENTATION_DATA);
    }

    pub(in crate::ui::retained_host::app) fn mark_render_and_presentation_dirty(&mut self) {
        self.invalidate_host(
            HostInvalidationMask::RENDER.union(HostInvalidationMask::PRESENTATION_DATA),
        );
    }
}
