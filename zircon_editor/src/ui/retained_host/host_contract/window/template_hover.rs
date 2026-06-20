use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};

use super::super::data::{
    HostPaneInteractionStateData, HostWindowPresentationData, PaneData, TemplatePaneNodeData,
};

pub(in crate::ui::retained_host::host_contract) fn apply_template_hover_to_presentation(
    presentation: &mut HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
) {
    if interaction.hovered_template_control_id.is_empty() {
        return;
    }
    apply_template_hover_to_nodes(&mut presentation.workbench_window_nodes, interaction);
    apply_template_hover_to_pane(
        &mut presentation.host_scene_data.document_dock.pane,
        interaction,
    );
    apply_template_hover_to_pane(
        &mut presentation.host_scene_data.left_dock.pane,
        interaction,
    );
    apply_template_hover_to_pane(
        &mut presentation.host_scene_data.right_dock.pane,
        interaction,
    );
    apply_template_hover_to_pane(
        &mut presentation.host_scene_data.bottom_dock.pane,
        interaction,
    );

    let mut floating_changed = false;
    let floating_windows: Vec<_> = (0..presentation
        .host_scene_data
        .floating_layer
        .floating_windows
        .row_count())
        .filter_map(|row| {
            presentation
                .host_scene_data
                .floating_layer
                .floating_windows
                .row_data(row)
        })
        .map(|mut window| {
            floating_changed |= apply_template_hover_to_pane(&mut window.active_pane, interaction);
            window
        })
        .collect();
    if floating_changed {
        presentation.host_scene_data.floating_layer.floating_windows =
            ModelRc::from(Rc::new(VecModel::from(floating_windows)));
    }
}

fn apply_template_hover_to_pane(
    pane: &mut PaneData,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    let nodes = match pane.kind.as_str() {
        "Hierarchy" => &mut pane.hierarchy.nodes,
        "Inspector" => &mut pane.inspector.nodes,
        "Console" => &mut pane.console.nodes,
        "Assets" => &mut pane.assets_activity.nodes,
        "AssetBrowser" => &mut pane.asset_browser.nodes,
        "Welcome" => &mut pane.welcome.nodes,
        "Project" | "UiComponentShowcase" => &mut pane.project_overview.nodes,
        "RuntimeDiagnostics" => &mut pane.runtime_diagnostics.nodes,
        "PerformanceTimeline" => &mut pane.performance_timeline.nodes,
        "ModulePlugins" => &mut pane.module_plugins.nodes,
        "BuildExport" => &mut pane.build_export.nodes,
        "GeneratedBottom" => &mut pane.generated_bottom.nodes,
        "UiAssetEditor" => &mut pane.ui_asset.nodes,
        "AnimationSequenceEditor" | "AnimationGraphEditor" => &mut pane.animation.nodes,
        _ => return false,
    };
    apply_template_hover_to_nodes(nodes, interaction)
}

fn apply_template_hover_to_nodes(
    nodes: &mut ModelRc<TemplatePaneNodeData>,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    let mut changed = false;
    let hovered = &interaction.hovered_template_control_id;
    let values: Vec<_> = (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .map(|mut node| {
            if node.control_id.as_str() == hovered.as_str() && !node.hovered {
                node.hovered = true;
                changed = true;
            }
            if node.control_id.as_str() == hovered.as_str() {
                changed |= apply_template_row_hover(&mut node, interaction);
            }
            node
        })
        .collect();
    if changed {
        *nodes = ModelRc::from(Rc::new(VecModel::from(values)));
    }
    changed
}

fn apply_template_row_hover(
    node: &mut TemplatePaneNodeData,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    match interaction.hovered_template_dispatch_kind.as_str() {
        "workbench_option" => {
            apply_option_row_hover(node, interaction.hovered_template_value_text.as_str())
        }
        "workbench_menu_item" => {
            apply_menu_row_hover(node, interaction.hovered_template_action_id.as_str())
        }
        _ => false,
    }
}

fn apply_option_row_hover(node: &mut TemplatePaneNodeData, option_id: &str) -> bool {
    if option_id.is_empty() || node.structured_options.row_count() == 0 {
        return false;
    }
    let mut changed = false;
    let options: Vec<_> = (0..node.structured_options.row_count())
        .filter_map(|row| node.structured_options.row_data(row))
        .map(|mut option| {
            let hovered = !option.disabled && option.id.as_str() == option_id;
            if option.hovered != hovered || option.focused || option.pressed {
                option.hovered = hovered;
                option.focused = false;
                option.pressed = false;
                changed = true;
            }
            option
        })
        .collect();
    if changed {
        node.structured_options = ModelRc::from(Rc::new(VecModel::from(options)));
    }
    changed
}

fn apply_menu_row_hover(node: &mut TemplatePaneNodeData, action_id: &str) -> bool {
    if action_id.is_empty() || node.structured_menu_items.row_count() == 0 {
        return false;
    }
    let mut changed = false;
    let items: Vec<_> = (0..node.structured_menu_items.row_count())
        .filter_map(|row| node.structured_menu_items.row_data(row))
        .map(|mut item| {
            let hovered = !item.disabled && !item.separator && item.action_id.as_str() == action_id;
            if item.hovered != hovered || item.focused || item.pressed {
                item.hovered = hovered;
                item.focused = false;
                item.pressed = false;
                changed = true;
            }
            item
        })
        .collect();
    if changed {
        node.structured_menu_items = ModelRc::from(Rc::new(VecModel::from(items)));
    }
    changed
}
