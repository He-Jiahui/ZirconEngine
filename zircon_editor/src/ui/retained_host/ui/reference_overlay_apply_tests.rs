use std::collections::BTreeMap;

use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::asset_editor::UiAssetEditorPanePresentation;
use crate::ui::retained_host::callback_dispatch::{
    BuiltinHostWindowTemplateBridge, BuiltinWorkbenchWindowTemplateSurfaceBridge,
};
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::retained_host::primitives::PhysicalSize;
use crate::ui::retained_host::UiHostWindow;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime_interface::ui::layout::UiSize;

const WORKBENCH_REFERENCE_WIDTH: u32 = 1672;
const WORKBENCH_REFERENCE_HEIGHT: u32 = 941;

#[test]
fn apply_presentation_keeps_workbench_reference_out_of_root_template_overlay() {
    let ui = reference_sized_host_window();
    apply_template_projection_from_host_template_bridge(&ui);

    let presentation = ui.get_host_presentation();
    assert_eq!(
        presentation.root_template_nodes.row_count(),
        0,
        "workbench shell must be composed from template components, not a final PNG overlay"
    );
}

#[test]
fn apply_presentation_carries_componentized_workbench_window_nodes_separately() {
    let ui = reference_sized_host_window();
    apply_template_projection_from_workbench_window_bridge(&ui);

    let presentation = ui.get_host_presentation();
    assert_eq!(
        presentation.root_template_nodes.row_count(),
        0,
        "componentized workbench nodes must not be promoted to the root overlay"
    );
    assert!(
        presentation.workbench_window_nodes.row_count() > 100,
        "workbench window projection should reach host presentation as component nodes"
    );
    let root = find_workbench_window_node(&presentation, "WorkbenchWindowRoot")
        .expect("workbench root control should be present");
    assert_eq!(root.role.as_str(), "Mount");
    assert_eq!(root.surface_variant.as_str(), "panel");
    assert_eq!(root.border_width, 1.0);
    assert_eq!(root.frame.width, WORKBENCH_REFERENCE_WIDTH as f32);
    assert_eq!(root.frame.height, WORKBENCH_REFERENCE_HEIGHT as f32);
    let primary = find_workbench_window_node(&presentation, "WorkbenchPrimaryButton")
        .expect("component drawer primary button should be present");
    assert_eq!(primary.role.as_str(), "Button");
    assert_eq!(primary.text.as_str(), "Primary");
    assert_eq!(primary.dispatch_kind.as_str(), "workbench");
    assert!(
        primary.action_id.as_str().starts_with("ComponentLab/"),
        "workbench controls should carry their template binding ids"
    );
    let selected_table_row = find_workbench_window_node(&presentation, "WorkbenchTableSelected")
        .expect("component drawer selected table row should be present");
    assert!(
        selected_table_row.selected,
        "component props selected=true should survive full-window projection"
    );
    assert!(
        !selected_table_row.checked,
        "selected visual state must not depend on checked state only"
    );
}

fn reference_sized_host_window() -> UiHostWindow {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window().set_size(PhysicalSize::new(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
    ));
    ui
}

fn apply_template_projection_from_host_template_bridge(ui: &UiHostWindow) {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);

    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("builtin workbench host template should project");
    let root_frames = template_bridge.root_shell_frames();
    let ui_asset_panes: BTreeMap<String, UiAssetEditorPanePresentation> = BTreeMap::new();
    let animation_panes: BTreeMap<String, AnimationEditorPanePresentation> = BTreeMap::new();
    let module_plugins =
        crate::ui::layouts::windows::workbench_host_window::ModulePluginsPaneViewData::default();
    let build_export =
        crate::ui::layouts::windows::workbench_host_window::BuildExportPaneViewData::default();
    let floating_window_projection_bundle = FloatingWindowProjectionBundle::default();

    super::apply_presentation_impl::apply_presentation(
        ui,
        &model,
        &chrome,
        &WorkbenchShellGeometry::default(),
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        &module_plugins,
        &build_export,
        Some(template_bridge.host_projection()),
        None,
        Some(&root_frames),
        &floating_window_projection_bundle,
        None,
    );
}

fn apply_template_projection_from_workbench_window_bridge(ui: &UiHostWindow) {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);

    let workbench_window_bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench window template should project");
    let ui_asset_panes: BTreeMap<String, UiAssetEditorPanePresentation> = BTreeMap::new();
    let animation_panes: BTreeMap<String, AnimationEditorPanePresentation> = BTreeMap::new();
    let module_plugins =
        crate::ui::layouts::windows::workbench_host_window::ModulePluginsPaneViewData::default();
    let build_export =
        crate::ui::layouts::windows::workbench_host_window::BuildExportPaneViewData::default();
    let floating_window_projection_bundle = FloatingWindowProjectionBundle::default();

    super::apply_presentation_impl::apply_presentation(
        ui,
        &model,
        &chrome,
        &WorkbenchShellGeometry::default(),
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        &module_plugins,
        &build_export,
        None,
        Some(workbench_window_bridge.host_projection()),
        None,
        &floating_window_projection_bundle,
        None,
    );
}

fn find_workbench_window_node(
    presentation: &crate::ui::retained_host::HostWindowPresentationData,
    control_id: &str,
) -> Option<crate::ui::retained_host::TemplatePaneNodeData> {
    (0..presentation.workbench_window_nodes.row_count())
        .filter_map(|row| presentation.workbench_window_nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
}
