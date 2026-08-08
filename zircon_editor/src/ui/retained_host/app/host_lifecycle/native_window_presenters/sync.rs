use std::collections::BTreeMap;
use std::rc::Weak;

use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::super::RetainedEditorHost;
use super::callbacks::wire_native_window_presenter_callbacks;
use super::presentation::apply_native_window_presenter_presentation;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::host_lifecycle) fn sync_native_window_presenters(
        &mut self,
        model: &WorkbenchViewModel,
        chrome: &EditorChromeSnapshot,
        geometry: &WorkbenchShellGeometry,
        preset_names: &[String],
        ui_asset_panes: &BTreeMap<String, crate::ui::asset_editor::UiAssetEditorPanePresentation>,
        animation_panes: &BTreeMap<
            String,
            crate::ui::animation_editor::AnimationEditorPanePresentation,
        >,
        runtime_diagnostics: &zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot,
        floating_window_projection_bundle: &FloatingWindowProjectionBundle,
    ) {
        let targets =
            self.collect_native_window_sync_targets(model, floating_window_projection_bundle);
        if self.sync_empty_native_window_targets(&targets) {
            return;
        }

        let pane_payloads = self.prepare_native_window_pane_payloads(model, chrome);
        let pane_template_runtime = if pane_payloads.has_component_showcase_runtime {
            &self.component_showcase_runtime
        } else {
            self.builtin_template_runtime.as_ref()
        };
        let active_preset_name = self.active_layout_preset.as_deref();
        let host_handle = self.self_handle.as_ref().and_then(Weak::upgrade);
        let viewport_toolbar_bridge = &mut self.viewport_toolbar_bridge;
        let source_generation = self.ui.get_host_presentation_generation().cursor();
        if let Err(error) = self.native_window_presenters.sync_targets_with_generation(
            &targets,
            source_generation,
            |ui, target| {
                wire_native_window_presenter_callbacks(ui, target, host_handle.as_ref());
            },
            |ui, target| {
                apply_native_window_presenter_presentation(
                    ui,
                    target,
                    model,
                    chrome,
                    geometry,
                    preset_names,
                    active_preset_name,
                    ui_asset_panes,
                    animation_panes,
                    runtime_diagnostics,
                    &pane_payloads,
                    floating_window_projection_bundle,
                    pane_template_runtime,
                    viewport_toolbar_bridge,
                );
            },
        ) {
            self.set_status_line(format!("Native window sync failed: {error}"));
        }
    }
}
