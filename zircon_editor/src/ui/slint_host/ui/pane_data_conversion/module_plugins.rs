use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    ModulePluginStatusViewData, ModulePluginsPaneViewData, PaneContentSize, PaneData,
};
use crate::ui::slint_host as host_contract;
use slint::Model;

const MODULE_PLUGIN_ROW_HEIGHT: f32 = 112.0;
const MODULE_PLUGIN_ROW_GAP: f32 = 8.0;
const MODULE_PLUGIN_ROW_PADDING: f32 = 8.0;
const MODULE_PLUGIN_BUTTON_HEIGHT: f32 = 24.0;
const MODULE_PLUGIN_BUTTON_GAP: f32 = 6.0;
const MODULE_PLUGIN_MIN_BUTTON_WIDTH: f32 = 56.0;
const MODULE_PLUGIN_MAX_BUTTON_WIDTH: f32 = 92.0;

pub(crate) fn to_host_contract_module_plugins_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::ModulePluginsPaneData {
    let native = &data.native_body.module_plugins;
    let mut nodes = module_plugins_template_projection(data, content_size).unwrap_or_default();
    nodes.extend(module_plugin_row_nodes(native, &nodes, content_size));

    host_contract::ModulePluginsPaneData {
        nodes: model_rc(nodes),
        plugins: super::map_model_rc(&native.plugins, to_host_contract_module_plugin_status),
        diagnostics: native.diagnostics.clone(),
    }
}

fn to_host_contract_module_plugin_status(
    data: ModulePluginStatusViewData,
) -> host_contract::ModulePluginStatusData {
    host_contract::ModulePluginStatusData {
        plugin_id: data.plugin_id,
        display_name: data.display_name,
        package_source: data.package_source,
        load_state: data.load_state,
        enabled: data.enabled,
        required: data.required,
        target_modes: data.target_modes,
        packaging: data.packaging,
        runtime_crate: data.runtime_crate,
        editor_crate: data.editor_crate,
        runtime_capabilities: data.runtime_capabilities,
        editor_capabilities: data.editor_capabilities,
        optional_features: data.optional_features,
        feature_action_label: data.feature_action_label,
        feature_action_id: data.feature_action_id,
        diagnostics: data.diagnostics,
        primary_action_label: data.primary_action_label,
        primary_action_id: data.primary_action_id,
        packaging_action_label: data.packaging_action_label,
        packaging_action_id: data.packaging_action_id,
        target_modes_action_label: data.target_modes_action_label,
        target_modes_action_id: data.target_modes_action_id,
        unload_action_label: data.unload_action_label,
        unload_action_id: data.unload_action_id,
        hot_reload_action_label: data.hot_reload_action_label,
        hot_reload_action_id: data.hot_reload_action_id,
    }
}

fn module_plugins_template_projection(
    data: &PaneData,
    content_size: PaneContentSize,
) -> Option<Vec<host_contract::TemplatePaneNodeData>> {
    let presentation = data.pane_presentation.as_ref()?;
    if !matches!(
        &presentation.body.payload,
        crate::ui::layouts::windows::workbench_host_window::PanePayload::ModulePluginsV1(_)
    ) {
        return None;
    }

    super::project_pane_template_nodes(&presentation.body, content_size)
}

fn module_plugin_row_nodes(
    data: &ModulePluginsPaneViewData,
    template_nodes: &[host_contract::TemplatePaneNodeData],
    content_size: PaneContentSize,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let list_frame = template_nodes
        .iter()
        .find(|node| {
            matches!(
                node.control_id.as_str(),
                "ModulePluginListSlotAnchor" | "ModulePluginListPanel"
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

    for row in 0..data.plugins.row_count() {
        let Some(plugin) = data.plugins.row_data(row) else {
            continue;
        };
        let plugin_id = plugin.plugin_id.to_string();
        let row_y = list_frame.y + row as f32 * (MODULE_PLUGIN_ROW_HEIGHT + MODULE_PLUGIN_ROW_GAP);
        let row_frame = host_contract::TemplateNodeFrameData {
            x: list_frame.x,
            y: row_y,
            width: list_width,
            height: MODULE_PLUGIN_ROW_HEIGHT,
        };
        let actions = module_plugin_row_actions(&plugin);
        let mut row_node = module_plugin_node(
            format!("module_plugin_row_{plugin_id}"),
            format!("ModulePluginRow.{plugin_id}"),
            "Panel",
            plugin.display_name.to_string(),
            row_frame.clone(),
        );
        row_node.surface_variant = "module-plugin-row".into();
        row_node.corner_radius = 6.0;
        row_node.border_width = 1.0;
        row_node.actions = model_rc(
            actions
                .iter()
                .map(|action| host_contract::TemplatePaneActionData {
                    label: action.label.clone().into(),
                    action_id: action.action_id.clone().into(),
                })
                .collect(),
        );
        nodes.push(row_node);

        nodes.push(module_plugin_node(
            format!("module_plugin_title_{plugin_id}"),
            format!("ModulePluginTitle.{plugin_id}"),
            "Label",
            plugin.display_name.to_string(),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x + MODULE_PLUGIN_ROW_PADDING,
                y: row_y + 8.0,
                width: (list_width - MODULE_PLUGIN_ROW_PADDING * 2.0).max(0.0),
                height: 20.0,
            },
        ));

        let mut meta = module_plugin_node(
            format!("module_plugin_meta_{plugin_id}"),
            format!("ModulePluginMeta.{plugin_id}"),
            "Label",
            format!(
                "{} | {} | {} | {}",
                plugin.package_source, plugin.load_state, plugin.packaging, plugin.target_modes
            ),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x + MODULE_PLUGIN_ROW_PADDING,
                y: row_y + 30.0,
                width: (list_width - MODULE_PLUGIN_ROW_PADDING * 2.0).max(0.0),
                height: 18.0,
            },
        );
        meta.text_tone = "muted".into();
        nodes.push(meta);

        let mut detail_y = row_y + 48.0;
        if !plugin.optional_features.is_empty() {
            let mut features = module_plugin_node(
                format!("module_plugin_features_{plugin_id}"),
                format!("ModulePluginFeatures.{plugin_id}"),
                "Label",
                plugin.optional_features.to_string(),
                host_contract::TemplateNodeFrameData {
                    x: list_frame.x + MODULE_PLUGIN_ROW_PADDING,
                    y: detail_y,
                    width: (list_width - MODULE_PLUGIN_ROW_PADDING * 2.0).max(0.0),
                    height: 16.0,
                },
            );
            features.text_tone = "muted".into();
            nodes.push(features);
            detail_y += 18.0;
        }

        if !plugin.diagnostics.is_empty() {
            let mut diagnostics = module_plugin_node(
                format!("module_plugin_diagnostics_{plugin_id}"),
                format!("ModulePluginDiagnostics.{plugin_id}"),
                "Label",
                plugin.diagnostics.to_string(),
                host_contract::TemplateNodeFrameData {
                    x: list_frame.x + MODULE_PLUGIN_ROW_PADDING,
                    y: detail_y,
                    width: (list_width - MODULE_PLUGIN_ROW_PADDING * 2.0).max(0.0),
                    height: 16.0,
                },
            );
            diagnostics.text_tone = "warning".into();
            nodes.push(diagnostics);
        }

        nodes.extend(module_plugin_action_button_nodes(
            &plugin_id,
            row_y,
            list_frame.x,
            list_width,
            &actions,
        ));
    }

    nodes
}

struct ModulePluginRowAction {
    label: String,
    action_id: String,
}

fn module_plugin_row_actions(plugin: &ModulePluginStatusViewData) -> Vec<ModulePluginRowAction> {
    [
        (
            plugin.primary_action_label.to_string(),
            plugin.primary_action_id.to_string(),
        ),
        (
            plugin.feature_action_label.to_string(),
            plugin.feature_action_id.to_string(),
        ),
        (
            plugin.packaging_action_label.to_string(),
            plugin.packaging_action_id.to_string(),
        ),
        (
            plugin.target_modes_action_label.to_string(),
            plugin.target_modes_action_id.to_string(),
        ),
        (
            plugin.unload_action_label.to_string(),
            plugin.unload_action_id.to_string(),
        ),
        (
            plugin.hot_reload_action_label.to_string(),
            plugin.hot_reload_action_id.to_string(),
        ),
    ]
    .into_iter()
    .filter(|(label, action_id)| !label.is_empty() && !action_id.is_empty())
    .map(|(label, action_id)| ModulePluginRowAction { label, action_id })
    .collect()
}

fn module_plugin_action_button_nodes(
    plugin_id: &str,
    row_y: f32,
    row_x: f32,
    row_width: f32,
    actions: &[ModulePluginRowAction],
) -> Vec<host_contract::TemplatePaneNodeData> {
    if actions.is_empty() {
        return Vec::new();
    }

    let available_width = (row_width - MODULE_PLUGIN_ROW_PADDING * 2.0).max(0.0);
    let gap_total = MODULE_PLUGIN_BUTTON_GAP * actions.len().saturating_sub(1) as f32;
    let button_width = ((available_width - gap_total) / actions.len() as f32).clamp(
        MODULE_PLUGIN_MIN_BUTTON_WIDTH,
        MODULE_PLUGIN_MAX_BUTTON_WIDTH,
    );
    let start_x = row_x + MODULE_PLUGIN_ROW_PADDING;
    let button_y =
        row_y + MODULE_PLUGIN_ROW_HEIGHT - MODULE_PLUGIN_ROW_PADDING - MODULE_PLUGIN_BUTTON_HEIGHT;

    actions
        .iter()
        .enumerate()
        .map(|(index, action)| {
            let mut node = module_plugin_node(
                format!("module_plugin_action_{plugin_id}_{index}"),
                "ModulePluginAction",
                "Button",
                compact_module_plugin_action_label(&action.label),
                host_contract::TemplateNodeFrameData {
                    x: start_x + index as f32 * (button_width + MODULE_PLUGIN_BUTTON_GAP),
                    y: button_y,
                    width: button_width,
                    height: MODULE_PLUGIN_BUTTON_HEIGHT,
                },
            );
            node.dispatch_kind = "module_plugin".into();
            node.action_id = action.action_id.clone().into();
            node.button_variant = "secondary".into();
            node.disabled = action.action_id.is_empty();
            node
        })
        .collect()
}

fn compact_module_plugin_action_label(label: &str) -> String {
    if label == "Cycle targets" {
        return "Targets".to_string();
    }
    if label.starts_with("Cycle ") {
        return "Package".to_string();
    }
    match label {
        "Hot Reload" => "Reload".to_string(),
        "Enable Deps" => "Deps".to_string(),
        "Enable Feature" => "Feature".to_string(),
        "Disable Feature" => "Feature Off".to_string(),
        other => other.to_string(),
    }
}

fn module_plugin_node(
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
        ModulePluginStatusViewData, ModulePluginsPaneViewData, PaneNativeBodyData,
    };

    #[test]
    fn module_plugins_pane_projects_visual_rows_and_action_buttons() {
        let pane = module_plugins_pane_fixture();
        let data = to_host_contract_module_plugins_pane_from_host_pane(
            &pane,
            PaneContentSize::new(480.0, 260.0),
        );

        assert_eq!(data.plugins.row_count(), 1);
        assert_eq!(
            data.plugins
                .row_data(0)
                .map(|plugin| plugin.plugin_id.to_string()),
            Some("physics".to_string())
        );

        let action_ids = (0..data.nodes.row_count())
            .filter_map(|row| data.nodes.row_data(row))
            .filter(|node| node.control_id.as_str() == "ModulePluginAction")
            .map(|node| node.action_id.to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            action_ids,
            vec![
                "Plugin.Disable.physics",
                "Plugin.Feature.Enable.physics.physics.raycast_queries",
                "Plugin.Packaging.Next.physics",
                "Plugin.TargetModes.Next.physics",
                "Plugin.Unload.physics",
                "Plugin.HotReload.physics",
            ]
        );

        let row_node = (0..data.nodes.row_count())
            .filter_map(|row| data.nodes.row_data(row))
            .find(|node| node.control_id.as_str() == "ModulePluginRow.physics")
            .expect("module plugin row should be projected");
        assert_eq!(row_node.actions.row_count(), 6);
        let feature_node = (0..data.nodes.row_count())
            .filter_map(|row| data.nodes.row_data(row))
            .find(|node| node.control_id.as_str() == "ModulePluginFeatures.physics")
            .expect("module plugin optional feature summary should be projected");
        assert_eq!(feature_node.text.to_string(), "Ray Cast Queries [ready]");
    }

    #[test]
    fn module_plugins_row_actions_skip_label_only_action_slots() {
        let mut pane = module_plugins_pane_fixture();
        let mut plugin = module_plugin_status_fixture();
        plugin.feature_action_id = "".into();
        pane.native_body.module_plugins.plugins = model_rc(vec![plugin]);

        let data = to_host_contract_module_plugins_pane_from_host_pane(
            &pane,
            PaneContentSize::new(480.0, 260.0),
        );

        let action_ids = (0..data.nodes.row_count())
            .filter_map(|row| data.nodes.row_data(row))
            .filter(|node| node.control_id.as_str() == "ModulePluginAction")
            .map(|node| node.action_id.to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            action_ids,
            vec![
                "Plugin.Disable.physics",
                "Plugin.Packaging.Next.physics",
                "Plugin.TargetModes.Next.physics",
                "Plugin.Unload.physics",
                "Plugin.HotReload.physics",
            ]
        );

        let row_node = (0..data.nodes.row_count())
            .filter_map(|row| data.nodes.row_data(row))
            .find(|node| node.control_id.as_str() == "ModulePluginRow.physics")
            .expect("module plugin row should be projected");
        assert_eq!(row_node.actions.row_count(), 5);
    }

    fn module_plugins_pane_fixture() -> PaneData {
        let module_plugins = module_plugins_fixture();
        PaneData {
            id: "editor.module_plugins#1".into(),
            slot: "left_bottom".into(),
            kind: "ModulePlugins".into(),
            title: "Plugin Manager".into(),
            icon_key: "plugin".into(),
            subtitle: "Project Plugins".into(),
            info: "Builtin and native plugin packages".into(),
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
                module_plugins,
                ..PaneNativeBodyData::default()
            },
            pane_presentation: None,
        }
    }

    fn module_plugins_fixture() -> ModulePluginsPaneViewData {
        ModulePluginsPaneViewData {
            plugins: model_rc(vec![module_plugin_status_fixture()]),
            diagnostics: "plugin catalog ready".into(),
        }
    }

    fn module_plugin_status_fixture() -> ModulePluginStatusViewData {
        ModulePluginStatusViewData {
            plugin_id: "physics".into(),
            display_name: "Physics".into(),
            package_source: "builtin".into(),
            load_state: "loaded".into(),
            enabled: true,
            required: false,
            target_modes: "editor, runtime".into(),
            packaging: "linked".into(),
            runtime_crate: "zircon_plugins_physics_runtime".into(),
            editor_crate: "zircon_plugins_physics_editor".into(),
            runtime_capabilities: "simulation".into(),
            editor_capabilities: "inspector".into(),
            optional_features: "Ray Cast Queries [ready]".into(),
            feature_action_label: "Enable Feature".into(),
            feature_action_id: "Plugin.Feature.Enable.physics.physics.raycast_queries".into(),
            diagnostics: "".into(),
            primary_action_label: "Disable".into(),
            primary_action_id: "Plugin.Disable.physics".into(),
            packaging_action_label: "Cycle linked".into(),
            packaging_action_id: "Plugin.Packaging.Next.physics".into(),
            target_modes_action_label: "Cycle targets".into(),
            target_modes_action_id: "Plugin.TargetModes.Next.physics".into(),
            unload_action_label: "Unload".into(),
            unload_action_id: "Plugin.Unload.physics".into(),
            hot_reload_action_label: "Hot Reload".into(),
            hot_reload_action_id: "Plugin.HotReload.physics".into(),
        }
    }
}
