use std::collections::BTreeMap;

use crate::ui::retained_host::UiHostWindow;
use crate::ui::retained_host::app::native_windows::{
    NativeFloatingWindowTarget, configure_native_floating_window_presentation,
};
use crate::ui::retained_host::callback_dispatch;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::retained_host::ui::apply_presentation;
use crate::ui::template_runtime::EditorUiHostRuntime;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::super::viewport_toolbar_projection::attach_viewport_toolbar_surface_frames_to_ui;
use super::payloads::NativeWindowPanePayloads;

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_native_window_presenter_presentation(
    ui: &UiHostWindow,
    target: &NativeFloatingWindowTarget,
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    preset_names: &[String],
    active_preset_name: Option<&str>,
    ui_asset_panes: &BTreeMap<String, crate::ui::asset_editor::UiAssetEditorPanePresentation>,
    animation_panes: &BTreeMap<
        String,
        crate::ui::animation_editor::AnimationEditorPanePresentation,
    >,
    runtime_diagnostics: &zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot,
    pane_payloads: &NativeWindowPanePayloads,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
    pane_template_runtime: &EditorUiHostRuntime,
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
) {
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
        &pane_payloads.module_plugins,
        &pane_payloads.build_export,
        None,
        None,
        callback_dispatch::BuiltinWorkbenchWindowLayoutFrames::default(),
        floating_window_projection_bundle,
        Some(pane_template_runtime),
    );
    attach_viewport_toolbar_surface_frames_to_ui(ui, viewport_toolbar_bridge, None);
    configure_native_floating_window_presentation(ui, target);
}
