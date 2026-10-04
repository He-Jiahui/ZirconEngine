use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    ModulePluginStatusViewData, ModulePluginsPaneViewData, PaneContentSize, PaneData, PanePayload,
};
use crate::ui::retained_host as host_contract;

use super::pane_template_runtime;

mod cache;

pub(crate) use cache::ModulePluginsPaneProjectionCache;
use cache::ModulePluginsPaneProjectionCacheKey;
const MODULE_PLUGIN_ROW_HEIGHT: f32 = 112.0;
const MODULE_PLUGIN_ROW_GAP: f32 = 8.0;
const MODULE_PLUGIN_ROW_PADDING: f32 = 8.0;
const MODULE_PLUGIN_BUTTON_HEIGHT: f32 = 24.0;
const MODULE_PLUGIN_BUTTON_GAP: f32 = 6.0;
const MODULE_PLUGIN_MIN_BUTTON_WIDTH: f32 = 56.0;
const MODULE_PLUGIN_MAX_BUTTON_WIDTH: f32 = 92.0;

#[cfg(test)]
fn to_host_contract_module_plugins_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::ModulePluginsPaneData {
    let mut cache = ModulePluginsPaneProjectionCache::default();
    to_host_contract_module_plugins_pane_from_host_pane_with_cache(data, content_size, &mut cache)
}

pub(crate) fn to_host_contract_module_plugins_pane_from_host_pane_with_cache(
    data: &PaneData,
    content_size: PaneContentSize,
    cache: &mut ModulePluginsPaneProjectionCache,
) -> host_contract::ModulePluginsPaneData {
    let native = &data.native_body.module_plugins;
    let cache_key = module_plugins_projection_cache_key(data, content_size);
    if let Some(cache_key) = cache_key {
        if let Some(pane) = cache.cached(
            data.id.as_str(),
            cache_key,
            &native.plugins,
            &native.diagnostics,
        ) {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.module_plugins.host_projection_cache_hit_count",
                1
            );
            zircon_runtime::profile_counter!(
                "editor",
                "ui.module_plugins.host_projection_source_row_count",
                0
            );
            return pane;
        }
    }
    zircon_runtime::profile_counter!(
        "editor",
        "ui.module_plugins.host_projection_cache_miss_count",
        1
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.module_plugins.host_projection_source_row_count",
        native.plugins.row_count()
    );
    let mut nodes = module_plugins_template_projection(data, content_size).unwrap_or_default();
    nodes.extend(module_plugin_row_nodes(native, &nodes, content_size));

    let pane = host_contract::ModulePluginsPaneData {
        nodes: model_rc(nodes),
        diagnostics: native.diagnostics.clone(),
    };
    if let Some(cache_key) = cache_key {
        cache.store(
            data.id.to_string(),
            cache_key,
            native.plugins.clone(),
            pane.clone(),
        );
    }
    pane
}

fn module_plugins_projection_cache_key(
    data: &PaneData,
    content_size: PaneContentSize,
) -> Option<ModulePluginsPaneProjectionCacheKey> {
    let (document_identity, uses_template) =
        match data.pane_presentation.as_ref().filter(|presentation| {
            matches!(&presentation.body.payload, PanePayload::ModulePluginsV1(_))
        }) {
            Some(presentation) => (
                pane_template_runtime(None)?
                    .retained_document_identity(&presentation.body.document_id)?,
                true,
            ),
            None => (0, false),
        };
    Some(ModulePluginsPaneProjectionCacheKey {
        document_identity,
        uses_template,
        width_bits: content_size.width.to_bits(),
        height_bits: content_size.height.to_bits(),
    })
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

    for (row, plugin) in data.plugins.iter().enumerate() {
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
                    label: action.label.into(),
                    action_id: action.action_id.into(),
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

struct ModulePluginRowAction<'a> {
    label: &'a str,
    action_id: &'a str,
}

fn module_plugin_row_actions(
    plugin: &ModulePluginStatusViewData,
) -> Vec<ModulePluginRowAction<'_>> {
    [
        (
            plugin.primary_action_label.as_str(),
            plugin.primary_action_id.as_str(),
        ),
        (
            plugin.feature_action_label.as_str(),
            plugin.feature_action_id.as_str(),
        ),
        (
            plugin.packaging_action_label.as_str(),
            plugin.packaging_action_id.as_str(),
        ),
        (
            plugin.target_modes_action_label.as_str(),
            plugin.target_modes_action_id.as_str(),
        ),
        (
            plugin.unload_action_label.as_str(),
            plugin.unload_action_id.as_str(),
        ),
        (
            plugin.hot_reload_action_label.as_str(),
            plugin.hot_reload_action_id.as_str(),
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
    actions: &[ModulePluginRowAction<'_>],
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
                compact_module_plugin_action_label(action.label),
                host_contract::TemplateNodeFrameData {
                    x: start_x + index as f32 * (button_width + MODULE_PLUGIN_BUTTON_GAP),
                    y: button_y,
                    width: button_width,
                    height: MODULE_PLUGIN_BUTTON_HEIGHT,
                },
            );
            node.dispatch_kind = "module_plugin".into();
            node.action_id = action.action_id.into();
            node.button_variant = "secondary".into();
            node.disabled = action.action_id.is_empty();
            node
        })
        .collect()
}

fn compact_module_plugin_action_label(label: &str) -> &str {
    if label == "Cycle targets" {
        return "Targets";
    }
    if label.starts_with("Cycle ") {
        return "Package";
    }
    match label {
        "Hot Reload" => "Reload",
        "Enable Deps" => "Deps",
        "Enable Feature" => "Feature",
        "Disable Feature" => "Feature Off",
        other => other,
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
#[path = "tests/module_plugins.rs"]
mod tests;
