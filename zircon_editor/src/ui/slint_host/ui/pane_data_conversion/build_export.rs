use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, BuildExportTargetViewData, PaneContentSize, PaneData,
};
use crate::ui::slint_host as host_contract;
use slint::Model;

const BUILD_EXPORT_ROW_HEIGHT: f32 = 118.0;
const BUILD_EXPORT_ROW_GAP: f32 = 8.0;
const BUILD_EXPORT_ROW_PADDING: f32 = 8.0;
const BUILD_EXPORT_BUTTON_HEIGHT: f32 = 24.0;
const BUILD_EXPORT_PRIMARY_BUTTON_WIDTH: f32 = 84.0;
const BUILD_EXPORT_SECONDARY_BUTTON_WIDTH: f32 = 72.0;
const BUILD_EXPORT_BUTTON_GAP: f32 = 6.0;

pub(crate) fn to_host_contract_build_export_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::BuildExportPaneData {
    let native = &data.native_body.build_export;
    let mut nodes = build_export_template_projection(data, content_size).unwrap_or_default();
    nodes.extend(build_export_target_row_nodes(native, &nodes, content_size));

    host_contract::BuildExportPaneData {
        nodes: model_rc(nodes),
        targets: super::map_model_rc(&native.targets, to_host_contract_build_export_target),
        diagnostics: native.diagnostics.clone(),
    }
}

fn to_host_contract_build_export_target(
    data: BuildExportTargetViewData,
) -> host_contract::BuildExportTargetData {
    host_contract::BuildExportTargetData {
        profile_name: data.profile_name,
        platform: data.platform,
        target_mode: data.target_mode,
        strategies: data.strategies,
        status: data.status,
        enabled_plugins: data.enabled_plugins,
        linked_runtime_crates: data.linked_runtime_crates,
        native_dynamic_packages: data.native_dynamic_packages,
        generated_files: data.generated_files,
        diagnostics: data.diagnostics,
        fatal: data.fatal,
    }
}

fn build_export_template_projection(
    data: &PaneData,
    content_size: PaneContentSize,
) -> Option<Vec<host_contract::TemplatePaneNodeData>> {
    let presentation = data.pane_presentation.as_ref()?;
    if !matches!(
        &presentation.body.payload,
        crate::ui::layouts::windows::workbench_host_window::PanePayload::BuildExportV1(_)
    ) {
        return None;
    }

    super::project_pane_template_nodes(&presentation.body, content_size)
}

fn build_export_target_row_nodes(
    data: &BuildExportPaneViewData,
    template_nodes: &[host_contract::TemplatePaneNodeData],
    content_size: PaneContentSize,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let list_frame = template_nodes
        .iter()
        .find(|node| {
            matches!(
                node.control_id.as_str(),
                "BuildExportTargetsSlotAnchor" | "BuildExportTargetsPanel"
            )
        })
        .map(|node| node.frame.clone())
        .unwrap_or_else(|| host_contract::TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: content_size.width.max(0.0),
            height: content_size.height.max(0.0),
        });
    let list_width = list_frame.width.max(content_size.width).max(0.0);
    let mut nodes = Vec::new();

    let targets = (0..data.targets.row_count())
        .filter_map(|row| data.targets.row_data(row))
        .collect::<Vec<_>>();
    let mut platform_counts = BTreeMap::new();
    for target in &targets {
        *platform_counts
            .entry(build_export_key(target.platform.as_str()))
            .or_insert(0usize) += 1;
    }
    let target_ids = targets
        .iter()
        .map(|target| {
            let platform_id = build_export_key(target.platform.as_str());
            build_export_target_id(
                &platform_id,
                target.profile_name.as_str(),
                platform_counts.get(&platform_id).copied().unwrap_or(0) > 1,
            )
        })
        .collect::<Vec<_>>();
    let mut target_id_counts = BTreeMap::new();
    for target_id in &target_ids {
        *target_id_counts.entry(target_id.clone()).or_insert(0usize) += 1;
    }
    let mut target_id_occurrences = BTreeMap::new();

    for (row, (target, mut target_id)) in targets.into_iter().zip(target_ids).enumerate() {
        if target_id_counts.get(&target_id).copied().unwrap_or(0) > 1 {
            let occurrence = target_id_occurrences
                .entry(target_id.clone())
                .and_modify(|count| *count += 1)
                .or_insert(1usize);
            target_id = format!("{target_id}.{occurrence}");
        }
        let node_id = format!("{row}_{target_id}");
        let row_y = list_frame.y + row as f32 * (BUILD_EXPORT_ROW_HEIGHT + BUILD_EXPORT_ROW_GAP);
        let actions = build_export_row_actions(&target);
        let mut row_node = build_export_node(
            format!("build_export_row_{node_id}"),
            format!("BuildExportRow.{target_id}"),
            "Panel",
            target.profile_name.to_string(),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x,
                y: row_y,
                width: list_width,
                height: BUILD_EXPORT_ROW_HEIGHT,
            },
        );
        row_node.surface_variant = if target.fatal {
            "diagnostic-error".into()
        } else {
            "build-export-row".into()
        };
        row_node.corner_radius = 6.0;
        row_node.border_width = 1.0;
        row_node.actions = model_rc(
            actions
                .iter()
                .map(|action| host_contract::TemplatePaneActionData {
                    label: action.label.into(),
                    action_id: action.action_id.clone().into(),
                })
                .collect(),
        );
        nodes.push(row_node);

        nodes.push(build_export_node(
            format!("build_export_title_{node_id}"),
            format!("BuildExportTitle.{target_id}"),
            "Label",
            format!("{} | {}", target.platform, target.status),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x + BUILD_EXPORT_ROW_PADDING,
                y: row_y + 8.0,
                width: (list_width - BUILD_EXPORT_ROW_PADDING * 2.0).max(0.0),
                height: 20.0,
            },
        ));

        let mut strategy = build_export_node(
            format!("build_export_strategy_{node_id}"),
            format!("BuildExportStrategy.{target_id}"),
            "Label",
            format!("{} | {}", target.target_mode, target.strategies),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x + BUILD_EXPORT_ROW_PADDING,
                y: row_y + 30.0,
                width: (list_width - BUILD_EXPORT_ROW_PADDING * 2.0).max(0.0),
                height: 18.0,
            },
        );
        strategy.text_tone = "muted".into();
        nodes.push(strategy);

        let mut counts = build_export_node(
            format!("build_export_counts_{node_id}"),
            format!("BuildExportCounts.{target_id}"),
            "Label",
            format!(
                "plugins {} | linked {} | native {} | files {}",
                target.enabled_plugins,
                target.linked_runtime_crates,
                target.native_dynamic_packages,
                target.generated_files
            ),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x + BUILD_EXPORT_ROW_PADDING,
                y: row_y + 48.0,
                width: (list_width - BUILD_EXPORT_ROW_PADDING * 2.0).max(0.0),
                height: 18.0,
            },
        );
        counts.text_tone = "muted".into();
        nodes.push(counts);

        let mut diagnostics = build_export_node(
            format!("build_export_diagnostics_{node_id}"),
            format!("BuildExportDiagnostics.{target_id}"),
            "Label",
            target.diagnostics.to_string(),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x + BUILD_EXPORT_ROW_PADDING,
                y: row_y + 66.0,
                width: (list_width - BUILD_EXPORT_ROW_PADDING * 2.0).max(0.0),
                height: 18.0,
            },
        );
        diagnostics.text_tone = if target.fatal { "danger" } else { "muted" }.into();
        nodes.push(diagnostics);

        nodes.extend(build_export_action_button_nodes(
            &node_id,
            row_y,
            list_frame.x,
            list_width,
            &actions,
        ));
    }

    nodes
}

fn build_export_target_id(
    platform_id: &str,
    profile_name: &str,
    duplicate_platform: bool,
) -> String {
    if duplicate_platform {
        format!("{platform_id}.{}", build_export_key(profile_name))
    } else {
        platform_id.to_string()
    }
}

fn build_export_key(value: &str) -> String {
    let key = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if key.is_empty() {
        "target".to_string()
    } else {
        key
    }
}

struct BuildExportRowAction {
    label: &'static str,
    action_id: String,
    variant: &'static str,
    disabled: bool,
    width: f32,
}

fn build_export_row_actions(target: &BuildExportTargetViewData) -> Vec<BuildExportRowAction> {
    let export_busy = matches!(
        target.status.as_str(),
        "Queued" | "Running" | "Cancel requested"
    );
    let (primary_label, primary_action_id) = if export_busy {
        (
            "Cancel",
            format!("BuildExport.Cancel.{}", target.profile_name),
        )
    } else {
        (
            "Export",
            format!("BuildExport.Execute.{}", target.profile_name),
        )
    };

    vec![
        BuildExportRowAction {
            label: primary_label,
            action_id: primary_action_id,
            variant: "primary",
            disabled: target.fatal && !export_busy,
            width: BUILD_EXPORT_PRIMARY_BUTTON_WIDTH,
        },
        BuildExportRowAction {
            label: "Choose",
            action_id: format!("BuildExport.ChooseOutput.{}", target.profile_name),
            variant: "secondary",
            disabled: false,
            width: BUILD_EXPORT_SECONDARY_BUTTON_WIDTH,
        },
        BuildExportRowAction {
            label: "Open",
            action_id: format!("BuildExport.RevealOutput.{}", target.profile_name),
            variant: "secondary",
            disabled: false,
            width: BUILD_EXPORT_SECONDARY_BUTTON_WIDTH,
        },
        BuildExportRowAction {
            label: "Default",
            action_id: format!("BuildExport.ClearOutput.{}", target.profile_name),
            variant: "secondary",
            disabled: false,
            width: BUILD_EXPORT_SECONDARY_BUTTON_WIDTH,
        },
    ]
}

fn build_export_action_button_nodes(
    platform_id: &str,
    row_y: f32,
    row_x: f32,
    row_width: f32,
    actions: &[BuildExportRowAction],
) -> Vec<host_contract::TemplatePaneNodeData> {
    if actions.is_empty() {
        return Vec::new();
    }

    let total_width = actions.iter().map(|action| action.width).sum::<f32>()
        + BUILD_EXPORT_BUTTON_GAP * actions.len().saturating_sub(1) as f32;
    let start_x = (row_x + row_width - BUILD_EXPORT_ROW_PADDING - total_width).max(row_x);
    let button_y =
        row_y + BUILD_EXPORT_ROW_HEIGHT - BUILD_EXPORT_ROW_PADDING - BUILD_EXPORT_BUTTON_HEIGHT;
    let mut cursor_x = start_x;

    actions
        .iter()
        .enumerate()
        .map(|(index, action)| {
            let mut node = build_export_node(
                format!("build_export_action_{platform_id}_{index}"),
                "BuildExportAction",
                "Button",
                action.label,
                host_contract::TemplateNodeFrameData {
                    x: cursor_x,
                    y: button_y,
                    width: action.width,
                    height: BUILD_EXPORT_BUTTON_HEIGHT,
                },
            );
            cursor_x += action.width + BUILD_EXPORT_BUTTON_GAP;
            node.dispatch_kind = "build_export".into();
            node.action_id = action.action_id.clone().into();
            node.button_variant = action.variant.into();
            node.disabled = action.disabled;
            node
        })
        .collect()
}

fn build_export_node(
    node_id: impl Into<String>,
    control_id: impl Into<String>,
    role: impl Into<String>,
    text: impl Into<String>,
    frame: host_contract::TemplateNodeFrameData,
) -> host_contract::TemplatePaneNodeData {
    host_contract::TemplatePaneNodeData {
        node_id: node_id.into().into(),
        control_id: control_id.into().into(),
        role: role.into().into(),
        text: text.into().into(),
        frame,
        ..host_contract::TemplatePaneNodeData::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::layouts::views::blank_viewport_chrome;
    use crate::ui::layouts::windows::workbench_host_window::{
        BuildExportPaneViewData, PaneNativeBodyData,
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
            Some("BuildExport.Execute.desktop_windows".into())
        );
        assert_eq!(
            row_node.actions.row_data(1).map(|action| action.action_id),
            Some("BuildExport.ChooseOutput.desktop_windows".into())
        );
        assert_eq!(
            row_node.actions.row_data(2).map(|action| action.action_id),
            Some("BuildExport.RevealOutput.desktop_windows".into())
        );
        assert_eq!(
            row_node.actions.row_data(3).map(|action| action.action_id),
            Some("BuildExport.ClearOutput.desktop_windows".into())
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
            "BuildExport.Execute.desktop_windows"
        );
        assert_eq!(
            actions[1].action_id.as_str(),
            "BuildExport.ChooseOutput.desktop_windows"
        );
        assert_eq!(
            actions[2].action_id.as_str(),
            "BuildExport.RevealOutput.desktop_windows"
        );
        assert_eq!(
            actions[3].action_id.as_str(),
            "BuildExport.ClearOutput.desktop_windows"
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
            "BuildExport.Cancel.desktop_linux"
        );
        let button = (0..data.nodes.row_count())
            .filter_map(|row| data.nodes.row_data(row))
            .find(|node| node.control_id.as_str() == "BuildExportAction")
            .expect("running desktop export action should be projected");
        assert_eq!(button.text.as_str(), "Cancel");
        assert_eq!(
            button.action_id.as_str(),
            "BuildExport.Cancel.desktop_linux"
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
}
