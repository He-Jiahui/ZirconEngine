use std::collections::BTreeMap;

use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::asset_editor::UiAssetEditorPanePresentation;
use crate::ui::retained_host::callback_dispatch::{
    BuiltinHostWindowTemplateBridge, BuiltinWorkbenchWindowTemplateSurfaceBridge,
    WorkbenchCommandPaletteOpenState,
};
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::retained_host::primitives::PhysicalSize;
use crate::ui::retained_host::UiHostWindow;
use crate::ui::template_runtime::RetainedUiHostProjection;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime_interface::ui::{component::UiValue, layout::UiSize};

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
    assert_eq!(primary.action_id.as_str(), "component_lab.button.primary");
    assert!(
        primary.binding_id.as_str().starts_with("ComponentLab/"),
        "workbench controls should carry their authored template binding ids"
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
    let inactive_module_tab = find_workbench_window_node(&presentation, "WorkbenchModuleScene")
        .expect("workbench module tab should be present");
    assert_eq!(inactive_module_tab.text.as_str(), "Scene");
    assert_eq!(
        inactive_module_tab.value_text.as_str(),
        "",
        "toggle state must not leak into module tab display text"
    );
    let active_module_tab = find_workbench_window_node(&presentation, "WorkbenchModuleEffect")
        .expect("selected workbench module tab should be present");
    assert_eq!(active_module_tab.text.as_str(), "Effect");
    assert_eq!(
        active_module_tab.value_text.as_str(),
        "",
        "selected toggle state must not render as boolean label text"
    );
}

#[test]
fn apply_presentation_projects_open_workbench_command_palette_rows_for_native_input() {
    let ui = reference_sized_host_window();
    let mut workbench_window_bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
            WORKBENCH_REFERENCE_WIDTH as f32,
            WORKBENCH_REFERENCE_HEIGHT as f32,
        ))
        .expect("componentized workbench window template should project");
    workbench_window_bridge
        .open_command_palette(WorkbenchCommandPaletteOpenState {
            commands: ui_string_values([
                "file.project.open|label=Open Project",
                "file.project.save|label=Save Project",
            ]),
            filtered_commands: ui_string_values(["file.project.open", "file.project.save"]),
            selected_command_id: "file.project.open".to_string(),
            focused_index: 0,
        })
        .expect("command palette state should apply to the workbench window bridge");
    apply_template_projection_from_workbench_window_projection(
        &ui,
        workbench_window_bridge.host_projection(),
    );

    let presentation = ui.get_host_presentation();
    let palette = find_workbench_window_node(&presentation, "WorkbenchCommandPalette")
        .expect("workbench command palette mount should be present");

    assert_eq!(palette.component_role.as_str(), "command-palette");
    assert!(palette.popup_open);
    assert_eq!(palette.structured_options.row_count(), 2);
    let open_project = palette
        .structured_options
        .row_data(0)
        .expect("open project command should project as a command palette row");
    assert_eq!(open_project.id.as_str(), "file.project.open");
    assert_eq!(open_project.label.as_str(), "Open Project");
    assert!(open_project.focused);
    assert!(open_project.selected);
    assert!(!open_project.disabled);

    let save_project = palette
        .structured_options
        .row_data(1)
        .expect("save project command should project as a command palette row");
    assert_eq!(save_project.id.as_str(), "file.project.save");
    assert!(!save_project.disabled);
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
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );

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
        crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames::default(),
        &floating_window_projection_bundle,
        None,
    );
}

fn apply_template_projection_from_workbench_window_bridge(ui: &UiHostWindow) {
    let workbench_window_bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench window template should project");
    apply_template_projection_from_workbench_window_projection(
        ui,
        workbench_window_bridge.host_projection(),
    );
}

fn apply_template_projection_from_workbench_window_projection(
    ui: &UiHostWindow,
    workbench_window_projection: &RetainedUiHostProjection,
) {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
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
        Some(workbench_window_projection),
        crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames::default(),
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

fn ui_string_values<const N: usize>(values: [&str; N]) -> UiValue {
    UiValue::Array(
        values
            .into_iter()
            .map(|value| UiValue::String(value.to_string()))
            .collect(),
    )
}
