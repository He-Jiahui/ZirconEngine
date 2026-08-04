use super::super::super::RetainedEditorHost;
use super::super::pane_payloads::HostLifecyclePanePayloads;
use crate::ui::retained_host::callback_dispatch;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::retained_host::ui::apply_presentation_with_template_v2_data;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

impl RetainedEditorHost {
    pub(super) fn apply_recompute_presentation(
        &mut self,
        model: &WorkbenchViewModel,
        chrome: &EditorChromeSnapshot,
        geometry: &WorkbenchShellGeometry,
        pane_payloads: &HostLifecyclePanePayloads,
        componentized_workbench_layout_frames: callback_dispatch::BuiltinWorkbenchWindowLayoutFrames,
        floating_window_projection_bundle: &FloatingWindowProjectionBundle,
    ) {
        zircon_runtime::profile_scope!("editor", "retained_host", "recompute_apply_presentation");
        let filtered_hierarchy_entries = self.filtered_hierarchy_entries(&chrome.scene_entries);
        let mut filtered_chrome = filtered_hierarchy_entries.map(|scene_entries| {
            let mut chrome = chrome.clone();
            chrome.scene_entries = scene_entries;
            chrome
        });
        let chrome = filtered_chrome.as_ref().unwrap_or(chrome);
        let _ = self.workbench_window_bridge.sync_from_chrome(chrome);
        let has_component_showcase_runtime =
            self.prepare_component_showcase_runtime_for_presentation(model);
        let pane_template_runtime = if has_component_showcase_runtime {
            &self.component_showcase_runtime
        } else {
            self.builtin_template_runtime.as_ref()
        };
        apply_presentation_with_template_v2_data(
            &self.ui,
            model,
            chrome,
            geometry,
            &pane_payloads.preset_names,
            self.active_layout_preset.as_deref(),
            &pane_payloads.ui_asset_panes,
            &pane_payloads.animation_panes,
            Some(&pane_payloads.runtime_diagnostics),
            &pane_payloads.module_plugins,
            &pane_payloads.build_export,
            &pane_payloads.template_v2_data,
            Some(self.template_bridge.host_projection()),
            Some(self.workbench_window_bridge.host_projection()),
            componentized_workbench_layout_frames,
            floating_window_projection_bundle,
            Some(pane_template_runtime),
            self.hierarchy_filter_query(),
        );
    }
}
