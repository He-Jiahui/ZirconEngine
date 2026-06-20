use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{PaneData, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract) fn pane_template_nodes(
    pane: &PaneData,
) -> Option<&ModelRc<TemplatePaneNodeData>> {
    match pane.kind.as_str() {
        "Hierarchy" => Some(&pane.hierarchy.nodes),
        "Inspector" => Some(&pane.inspector.nodes),
        "Console" => Some(&pane.console.nodes),
        "Assets" => Some(&pane.assets_activity.nodes),
        "AssetBrowser" => Some(&pane.asset_browser.nodes),
        "Welcome" => Some(&pane.welcome.nodes),
        "Project" | "UiComponentShowcase" => Some(&pane.project_overview.nodes),
        "RuntimeDiagnostics" => Some(&pane.runtime_diagnostics.nodes),
        "PerformanceTimeline" => Some(&pane.performance_timeline.nodes),
        "ModulePlugins" => Some(&pane.module_plugins.nodes),
        "BuildExport" => Some(&pane.build_export.nodes),
        "GeneratedBottom" => Some(&pane.generated_bottom.nodes),
        "UiAssetEditor" => Some(&pane.ui_asset.nodes),
        "AnimationSequenceEditor" | "AnimationGraphEditor" => Some(&pane.animation.nodes),
        _ => None,
    }
}
