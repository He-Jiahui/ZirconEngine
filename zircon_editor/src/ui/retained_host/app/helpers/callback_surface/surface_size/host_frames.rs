use crate::ui::workbench::snapshot::ViewContentKind;
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::super::super::RetainedEditorHost;

mod source_window;
mod workbench;

use workbench::resolve_workbench_host_frame_backed_size_for_kind;

impl RetainedEditorHost {
    pub(super) fn resolve_host_frame_backed_size_for_kind(
        &self,
        kind: ViewContentKind,
    ) -> Option<UiSize> {
        self.resolve_callback_source_window_host_frame_backed_size()
            .or_else(|| resolve_workbench_host_frame_backed_size_for_kind(self, kind))
    }
}
