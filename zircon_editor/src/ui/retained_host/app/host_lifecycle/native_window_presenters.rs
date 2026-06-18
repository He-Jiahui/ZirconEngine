use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

use crate::ui::retained_host::callback_dispatch;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::retained_host::primitives::CloseRequestResponse;
use crate::ui::retained_host::ui::apply_presentation;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::{EditorChromeSnapshot, ViewContentKind};

use super::super::callback_wiring::wire_callbacks;
use super::super::native_windows::{
    collect_native_floating_window_targets, configure_native_floating_window_presentation,
};
use super::super::pane_payload_visibility;
use super::super::viewport_toolbar_projection::attach_viewport_toolbar_surface_frames_to_ui;
use super::super::RetainedEditorHost;

impl RetainedEditorHost {
    pub(super) fn sync_native_window_presenters(
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
            collect_native_floating_window_targets(model, floating_window_projection_bundle);
        if targets.is_empty() {
            if let Err(error) =
                self.native_window_presenters
                    .sync_targets(&targets, |_, _| {}, |_, _| {})
            {
                self.set_status_line(format!("Native window sync failed: {error}"));
            }
            return;
        }

        let module_plugins = if pane_payload_visibility::should_collect_payload_for_kind(
            model,
            ViewContentKind::ModulePlugins,
        ) {
            self.module_plugins_pane_data(chrome)
        } else {
            crate::ui::layouts::windows::workbench_host_window::ModulePluginsPaneViewData::default()
        };
        let build_export = if pane_payload_visibility::should_collect_payload_for_kind(
            model,
            ViewContentKind::BuildExport,
        ) {
            self.build_export_pane_data(chrome)
        } else {
            crate::ui::layouts::windows::workbench_host_window::BuildExportPaneViewData::default()
        };
        let has_component_showcase_runtime =
            self.prepare_component_showcase_runtime_for_presentation(model);
        let pane_template_runtime = if has_component_showcase_runtime {
            &self.component_showcase_runtime
        } else {
            self.builtin_template_runtime.as_ref()
        };
        let active_preset_name = self.active_layout_preset.as_deref();
        let host_handle = self.self_handle.as_ref().and_then(Weak::upgrade);
        let viewport_toolbar_bridge = &mut self.viewport_toolbar_bridge;
        if let Err(error) = self.native_window_presenters.sync_targets(
            &targets,
            |ui, target| {
                if let Some(host) = host_handle.as_ref() {
                    wire_callbacks(ui, host);
                    let host_weak: Weak<RefCell<RetainedEditorHost>> = Rc::downgrade(host);
                    let window_id = target.window_id.clone();
                    ui.window().on_close_requested(move || {
                        if let Some(host) = host_weak.upgrade() {
                            host.borrow_mut()
                                .native_floating_window_close_requested(&window_id)
                        } else {
                            CloseRequestResponse::KeepWindowShown
                        }
                    });
                }
            },
            |ui, target| {
                apply_presentation(
                    ui,
                    model,
                    chrome,
                    geometry,
                    preset_names,
                    active_preset_name,
                    ui_asset_panes,
                    animation_panes,
                    Some(runtime_diagnostics),
                    &module_plugins,
                    &build_export,
                    None,
                    None,
                    callback_dispatch::BuiltinWorkbenchWindowLayoutFrames::default(),
                    floating_window_projection_bundle,
                    Some(pane_template_runtime),
                );
                attach_viewport_toolbar_surface_frames_to_ui(ui, viewport_toolbar_bridge, None);
                configure_native_floating_window_presentation(ui, target);
            },
        ) {
            self.set_status_line(format!("Native window sync failed: {error}"));
        }
    }
}
