use super::super::build_export_wizard_panel::{
    build_export_wizard_panel_nodes, EXPORT_WIZARD_PANEL_DISPATCH_KIND,
};
use super::*;
use crate::ui::host::{
    DESKTOP_EXPORT_CANCEL_BINDING_ID, DESKTOP_EXPORT_CANCEL_BUTTON, DESKTOP_EXPORT_STAGE_ROWS_SLOT,
    DESKTOP_EXPORT_START_BINDING_ID, DESKTOP_EXPORT_START_BUTTON,
};
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::blank_viewport_chrome;
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, BuildExportTargetViewData, PaneContentSize, PaneData,
    PaneNativeBodyData,
};

#[test]
fn build_export_pane_projects_desktop_target_rows() {
    let pane = PaneData {
        id: "editor.build_export_desktop#1".into(),
        slot: "bottom_right".into(),
        kind: "BuildExport".into(),
        title: "Desktop Export".into(),
        icon_key: "build-export".into(),
        subtitle: "Desktop Targets".into(),
        info: "Windows, Linux, and macOS export plans".into(),
        show_empty: false,
        empty_title: "".into(),
        empty_body: "".into(),
        primary_action_label: "".into(),
        primary_action_id: "".into(),
        secondary_action_label: "".into(),
        secondary_action_id: "".into(),
        secondary_hint: "".into(),
        show_toolbar: false,
        viewport: blank_viewport_chrome(),
        native_body: PaneNativeBodyData {
            build_export: BuildExportPaneViewData {
                targets: model_rc(vec![BuildExportTargetViewData {
                    profile_name: "desktop_windows".into(),
                    platform: "Windows".into(),
                    target_mode: "ClientRuntime".into(),
                    strategies: "SourceTemplate, LibraryEmbed, NativeDynamic".into(),
                    status: "Ready".into(),
                    enabled_plugins: "2".into(),
                    linked_runtime_crates: "1".into(),
                    native_dynamic_packages: "1".into(),
                    generated_files: "5".into(),
                    diagnostics: "native plugin package ready".into(),
                    fatal: false,
                }]),
                diagnostics: "export ready".into(),
                ..BuildExportPaneViewData::default()
            },
            ..PaneNativeBodyData::default()
        },
        pane_presentation: None,
    };
    let data = to_host_contract_build_export_pane_from_host_pane(
        &pane,
        PaneContentSize::new(520.0, 180.0),
    );

    assert_eq!(data.targets.row_count(), 1);
    let row_node = (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .find(|node| node.control_id.as_str() == "BuildExportRow.windows")
        .expect("desktop export target row should be projected");
    assert_eq!(row_node.text.to_string(), "desktop_windows");
    assert_eq!(row_node.actions.row_count(), 4);
    assert_eq!(
        row_node.actions.row_data(0).map(|action| action.action_id),
        Some("workbench.build_export.execute.desktop_windows".into())
    );
    assert_eq!(
        row_node.actions.row_data(1).map(|action| action.action_id),
        Some("workbench.build_export.output.choose.desktop_windows".into())
    );
    assert_eq!(
        row_node.actions.row_data(2).map(|action| action.action_id),
        Some("workbench.build_export.output.reveal.desktop_windows".into())
    );
    assert_eq!(
        row_node.actions.row_data(3).map(|action| action.action_id),
        Some("workbench.build_export.output.clear.desktop_windows".into())
    );
    let counts = (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .find(|node| node.control_id.as_str() == "BuildExportCounts.windows")
        .expect("desktop export target counts should be projected");
    assert!(counts.text.to_string().contains("native 1"));
    let diagnostics = (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .find(|node| node.control_id.as_str() == "BuildExportDiagnostics.windows")
        .expect("desktop export target diagnostics should be projected");
    assert_eq!(diagnostics.text.as_str(), "native plugin package ready");
    let actions = (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .filter(|node| node.control_id.as_str() == "BuildExportAction")
        .collect::<Vec<_>>();
    assert_eq!(actions.len(), 4);
    assert_eq!(
        actions[0].action_id.as_str(),
        "workbench.build_export.execute.desktop_windows"
    );
    assert_eq!(
        actions[1].action_id.as_str(),
        "workbench.build_export.output.choose.desktop_windows"
    );
    assert_eq!(
        actions[2].action_id.as_str(),
        "workbench.build_export.output.reveal.desktop_windows"
    );
    assert_eq!(
        actions[3].action_id.as_str(),
        "workbench.build_export.output.clear.desktop_windows"
    );
    assert!(!actions[0].disabled);
}

#[test]
fn build_export_running_target_projects_cancel_action() {
    let pane = PaneData {
        id: "editor.build_export_desktop#1".into(),
        slot: "bottom_right".into(),
        kind: "BuildExport".into(),
        title: "Desktop Export".into(),
        icon_key: "build-export".into(),
        subtitle: "Desktop Targets".into(),
        info: "Windows, Linux, and macOS export plans".into(),
        show_empty: false,
        empty_title: "".into(),
        empty_body: "".into(),
        primary_action_label: "".into(),
        primary_action_id: "".into(),
        secondary_action_label: "".into(),
        secondary_action_id: "".into(),
        secondary_hint: "".into(),
        show_toolbar: false,
        viewport: blank_viewport_chrome(),
        native_body: PaneNativeBodyData {
            build_export: BuildExportPaneViewData {
                targets: model_rc(vec![BuildExportTargetViewData {
                    profile_name: "desktop_linux".into(),
                    platform: "Linux".into(),
                    target_mode: "ClientRuntime".into(),
                    strategies: "SourceTemplate, LibraryEmbed, NativeDynamic".into(),
                    status: "Running".into(),
                    enabled_plugins: "2".into(),
                    linked_runtime_crates: "1".into(),
                    native_dynamic_packages: "1".into(),
                    generated_files: "5".into(),
                    diagnostics: "Progress: export backend is running".into(),
                    fatal: false,
                }]),
                diagnostics: "export ready".into(),
                ..BuildExportPaneViewData::default()
            },
            ..PaneNativeBodyData::default()
        },
        pane_presentation: None,
    };
    let data = to_host_contract_build_export_pane_from_host_pane(
        &pane,
        PaneContentSize::new(520.0, 180.0),
    );

    let row_node = (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .find(|node| node.control_id.as_str() == "BuildExportRow.linux")
        .expect("running desktop export target row should be projected");
    assert_eq!(row_node.actions.row_count(), 4);
    let row_action = row_node
        .actions
        .row_data(0)
        .expect("running row should expose cancel action");
    assert_eq!(row_action.label.as_str(), "Cancel");
    assert_eq!(
        row_action.action_id.as_str(),
        "workbench.build_export.cancel.desktop_linux"
    );
    let button = (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .find(|node| node.control_id.as_str() == "BuildExportAction")
        .expect("running desktop export action should be projected");
    assert_eq!(button.text.as_str(), "Cancel");
    assert_eq!(
        button.action_id.as_str(),
        "workbench.build_export.cancel.desktop_linux"
    );
}

#[test]
fn build_export_empty_diagnostics_still_projects_diagnostics_node() {
    let pane = build_export_pane_fixture(vec![build_export_target_fixture(
        "desktop_windows",
        "Windows",
        "Ready",
        "",
        false,
    )]);
    let data = to_host_contract_build_export_pane_from_host_pane(
        &pane,
        PaneContentSize::new(520.0, 180.0),
    );

    let diagnostics = (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .find(|node| node.control_id.as_str() == "BuildExportDiagnostics.windows")
        .expect("empty diagnostics node should still be projected");
    assert_eq!(diagnostics.text.as_str(), "");
}

#[test]
fn build_export_wizard_panel_nodes_project_retained_export_wizard_panel() {
    let pane = build_export_pane_fixture(vec![build_export_target_fixture(
        "desktop_windows",
        "Windows",
        "Ready",
        "native plugin package ready",
        false,
    )]);
    let nodes = build_export_wizard_panel_nodes(
        &pane.native_body.build_export,
        PaneContentSize::new(960.0, 420.0),
    )
    .expect("desktop export wizard panel should project");

    assert!(nodes
        .iter()
        .any(|node| node.control_id.as_str() == "DesktopExportRoot"));
    assert!(!nodes
        .iter()
        .any(|node| node.control_id.as_str().starts_with("BuildExportRow.")));

    let start_button = nodes
        .iter()
        .find(|node| node.control_id.as_str() == DESKTOP_EXPORT_START_BUTTON)
        .expect("start button should project");
    assert!(!start_button.disabled);
    assert_eq!(
        start_button.binding_id.as_str(),
        DESKTOP_EXPORT_START_BINDING_ID
    );
    assert_eq!(
        start_button.action_id.as_str(),
        "workbench.build_export.execute.desktop_windows"
    );
    assert_eq!(
        start_button.dispatch_kind.as_str(),
        EXPORT_WIZARD_PANEL_DISPATCH_KIND
    );

    let cancel_button = nodes
        .iter()
        .find(|node| node.control_id.as_str() == DESKTOP_EXPORT_CANCEL_BUTTON)
        .expect("cancel button should project");
    assert!(cancel_button.disabled);
    assert_eq!(
        cancel_button.binding_id.as_str(),
        DESKTOP_EXPORT_CANCEL_BINDING_ID
    );
    assert_eq!(
        cancel_button.action_id.as_str(),
        "workbench.build_export.cancel.desktop_windows"
    );
    assert_eq!(cancel_button.dispatch_kind.as_str(), "");

    let stage_rows = nodes
        .iter()
        .filter(|node| {
            node.control_id
                .as_str()
                .starts_with(&format!("{DESKTOP_EXPORT_STAGE_ROWS_SLOT}.stage."))
        })
        .collect::<Vec<_>>();
    assert!(!stage_rows.is_empty());
    assert!(stage_rows
        .iter()
        .any(|node| node.text.as_str().contains("Validate")));
}

#[test]
fn build_export_wizard_panel_nodes_respect_target_strategy_list() {
    let mut target = build_export_target_fixture(
        "browser_webgpu",
        "WebGPU",
        "Ready",
        "browser export plan ready",
        false,
    );
    target.strategies = "SourceTemplate, LibraryEmbed".into();
    target.native_dynamic_packages = "0".into();
    let pane = build_export_pane_fixture(vec![target]);
    let nodes = build_export_wizard_panel_nodes(
        &pane.native_body.build_export,
        PaneContentSize::new(960.0, 420.0),
    )
    .expect("browser export wizard panel should project");

    let stage_rows = nodes
        .iter()
        .filter(|node| {
            node.control_id
                .as_str()
                .starts_with(&format!("{DESKTOP_EXPORT_STAGE_ROWS_SLOT}.stage."))
        })
        .collect::<Vec<_>>();
    assert!(stage_rows
        .iter()
        .any(|node| node.text.as_str().contains("SourceTemplate")));
    assert!(stage_rows
        .iter()
        .any(|node| node.text.as_str().contains("PlatformBundle")));
    assert!(!stage_rows
        .iter()
        .any(|node| node.text.as_str().contains("NativeDynamic")));
}

#[test]
fn build_export_duplicate_platform_profiles_get_unique_projection_ids() {
    let pane = build_export_pane_fixture(vec![
        build_export_target_fixture("desktop_windows", "Windows", "Ready", "", false),
        build_export_target_fixture("desktop_windows", "Windows", "Ready", "", false),
        build_export_target_fixture("desktop-windows", "Windows", "Ready", "", false),
    ]);
    let data = to_host_contract_build_export_pane_from_host_pane(
        &pane,
        PaneContentSize::new(520.0, 260.0),
    );

    let row_control_ids = (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .filter(|node| {
            node.control_id
                .as_str()
                .starts_with("BuildExportRow.windows")
        })
        .map(|node| node.control_id.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        row_control_ids,
        vec![
            "BuildExportRow.windows.desktop_windows.1",
            "BuildExportRow.windows.desktop_windows.2",
            "BuildExportRow.windows.desktop_windows.3",
        ]
    );
}

fn build_export_pane_fixture(targets: Vec<BuildExportTargetViewData>) -> PaneData {
    PaneData {
        id: "editor.build_export_desktop#1".into(),
        slot: "bottom_right".into(),
        kind: "BuildExport".into(),
        title: "Desktop Export".into(),
        icon_key: "build-export".into(),
        subtitle: "Desktop Targets".into(),
        info: "Windows, Linux, and macOS export plans".into(),
        show_empty: false,
        empty_title: "".into(),
        empty_body: "".into(),
        primary_action_label: "".into(),
        primary_action_id: "".into(),
        secondary_action_label: "".into(),
        secondary_action_id: "".into(),
        secondary_hint: "".into(),
        show_toolbar: false,
        viewport: blank_viewport_chrome(),
        native_body: PaneNativeBodyData {
            build_export: BuildExportPaneViewData {
                targets: model_rc(targets),
                diagnostics: "export ready".into(),
                ..BuildExportPaneViewData::default()
            },
            ..PaneNativeBodyData::default()
        },
        pane_presentation: None,
    }
}

fn build_export_target_fixture(
    profile_name: &str,
    platform: &str,
    status: &str,
    diagnostics: &str,
    fatal: bool,
) -> BuildExportTargetViewData {
    BuildExportTargetViewData {
        profile_name: profile_name.into(),
        platform: platform.into(),
        target_mode: "ClientRuntime".into(),
        strategies: "SourceTemplate, LibraryEmbed, NativeDynamic".into(),
        status: status.into(),
        enabled_plugins: "2".into(),
        linked_runtime_crates: "1".into(),
        native_dynamic_packages: "1".into(),
        generated_files: "5".into(),
        diagnostics: diagnostics.into(),
        fatal,
    }
}
