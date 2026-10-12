use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::MainPageId;
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::super::super::super::RetainedEditorHost;
use super::super::frame::{frame_size, is_valid_size};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn resolve_floating_window_content_frame_for_window(
        &self,
        window_id: &MainPageId,
    ) -> Option<ShellFrame> {
        self.floating_window_projection_bundle
            .content_frame(window_id)
    }

    pub(super) fn resolve_callback_source_window_host_frame_backed_size(&self) -> Option<UiSize> {
        let window_id = self.callback_source_window.as_ref()?;
        self.resolve_floating_window_content_frame_for_window(window_id)
            .and_then(frame_size)
            .or_else(|| self.resolve_native_floating_window_content_size_for_window(window_id))
    }

    fn resolve_native_floating_window_content_size_for_window(
        &self,
        window_id: &MainPageId,
    ) -> Option<UiSize> {
        let window = self.native_window_presenters.window(window_id)?;
        let generation = window.get_host_presentation_generation();
        let bounds = &generation.structure().host_shell.native_window_bounds;
        let size = UiSize::new(
            bounds.width.max(0.0),
            (bounds.height
                - self.chrome_metrics.document_header_height
                - self.chrome_metrics.separator_thickness)
                .max(0.0),
        );
        is_valid_size(size).then_some(size)
    }
}
