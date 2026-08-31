use std::sync::Arc;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::BuildExportPaneViewData;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::RetainedEditorHost;
use cache::BuildExportBaseLookup;

pub(super) mod cache;
mod targets;

impl RetainedEditorHost {
    pub(super) fn build_export_pane_data(
        &self,
        chrome: &EditorChromeSnapshot,
    ) -> BuildExportPaneViewData {
        let project_path = std::path::Path::new(&chrome.project_path);
        let base_lookup = self
            .desktop_export_wizard_sessions
            .projection_cache()
            .borrow_mut()
            .lookup_base(project_path);
        let (base_revision, base) = match base_lookup {
            BuildExportBaseLookup::Hit {
                revision,
                projection,
            } => (Some(revision), projection),
            BuildExportBaseLookup::Miss(build_token) => {
                let base = Arc::new(targets::rebuild_export_targets(self, chrome));
                let revision = self
                    .desktop_export_wizard_sessions
                    .projection_cache()
                    .borrow_mut()
                    .store_base(build_token, Arc::clone(&base));
                (revision, base)
            }
        };
        let overlay_generation = self
            .desktop_export_wizard_sessions
            .projection_overlay_generation();
        if let Some(cached) = base_revision.and_then(|revision| {
            self.desktop_export_wizard_sessions
                .projection_cache()
                .borrow()
                .cached_rendered(revision, overlay_generation)
        }) {
            return cached;
        }

        let targets = targets::apply_export_target_overlays(self, base.as_ref());
        let wizard_view_model = targets.first().and_then(|target| {
            self.desktop_export_wizard_sessions
                .view_model(target.preset_name.as_str())
                .cloned()
        });

        let pane = BuildExportPaneViewData {
            targets: model_rc(targets),
            diagnostics: base.diagnostics.join("\n").into(),
            wizard_view_model,
        };
        if let Some(revision) = base_revision {
            self.desktop_export_wizard_sessions
                .projection_cache()
                .borrow_mut()
                .store_rendered(revision, overlay_generation, pane.clone());
        }
        pane
    }
}
