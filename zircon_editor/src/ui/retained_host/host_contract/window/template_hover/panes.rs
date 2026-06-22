use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};

use super::super::super::data::{
    HostPaneInteractionStateData, HostWindowPresentationData, PaneData,
};
use super::nodes::apply_template_hover_to_nodes;

pub(super) fn apply_template_hover_to_dock_panes(
    presentation: &mut HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
) {
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
}

pub(super) fn apply_template_hover_to_floating_panes(
    presentation: &mut HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
) {
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
