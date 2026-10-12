use crate::ui::retained_host::app::native_windows::{
    collect_native_floating_window_targets, NativeFloatingWindowTarget,
};
use crate::ui::retained_host::app::RetainedEditorHost;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::workbench::model::WorkbenchViewModel;

impl RetainedEditorHost {
    pub(super) fn collect_native_window_sync_targets(
        &self,
        model: &WorkbenchViewModel,
        floating_window_projection_bundle: &FloatingWindowProjectionBundle,
    ) -> Vec<NativeFloatingWindowTarget> {
        collect_native_floating_window_targets(model, floating_window_projection_bundle)
    }

    pub(super) fn sync_empty_native_window_targets(
        &mut self,
        targets: &[NativeFloatingWindowTarget],
    ) -> bool {
        if !targets.is_empty() {
            return false;
        }
        if let Err(error) =
            self.native_window_presenters
                .sync_targets(targets, |_, _| {}, |_, _| {})
        {
            self.set_status_line(format!("Native window sync failed: {error}"));
        }
        true
    }
}
