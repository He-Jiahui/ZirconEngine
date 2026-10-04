use std::sync::Arc;

use crate::ui::retained_host::host_contract::surface_hit_test::HostPaneTemplateHitIndex;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

use super::super::{TemplatePaneNodeData, UiAssetEditorPaneData, WelcomePaneData};
use super::{
    animation::AnimationEditorPaneData,
    basic::{
        AssetBrowserPaneData, AssetsActivityPaneData, ConsolePaneData, GeneratedBottomPaneData,
        ProjectOverviewPaneData, TemplateV2PaneData,
    },
    build_export::BuildExportPaneData,
    hierarchy::HierarchyPaneData,
    inspector::InspectorPaneData,
    module_plugins::ModulePluginsPaneData,
    performance_timeline::PerformanceTimelinePaneData,
    runtime_diagnostics::RuntimeDiagnosticsPaneData,
    viewport::SceneViewportChromeData,
};

#[derive(Clone, Default)]
pub(crate) struct PaneData {
    pub id: SharedString,
    pub slot: SharedString,
    pub kind: SharedString,
    pub title: SharedString,
    pub icon_key: SharedString,
    pub subtitle: SharedString,
    pub info: SharedString,
    pub show_empty: bool,
    pub empty_title: SharedString,
    pub empty_body: SharedString,
    pub primary_action_label: SharedString,
    pub primary_action_id: SharedString,
    pub secondary_action_label: SharedString,
    pub secondary_action_id: SharedString,
    pub secondary_hint: SharedString,
    pub show_toolbar: bool,
    pub body_surface_frame: Option<Arc<UiSurfaceFrame>>,
    pub body_template_hit_index: Option<Arc<HostPaneTemplateHitIndex>>,
    pub welcome: WelcomePaneData,
    pub viewport: SceneViewportChromeData,
    pub hierarchy: HierarchyPaneData,
    pub inspector: InspectorPaneData,
    pub console: ConsolePaneData,
    pub assets_activity: AssetsActivityPaneData,
    pub asset_browser: AssetBrowserPaneData,
    pub project_overview: ProjectOverviewPaneData,
    pub template_v2: TemplateV2PaneData,
    pub runtime_diagnostics: RuntimeDiagnosticsPaneData,
    pub performance_timeline: PerformanceTimelinePaneData,
    pub module_plugins: ModulePluginsPaneData,
    pub build_export: BuildExportPaneData,
    pub generated_bottom: GeneratedBottomPaneData,
    pub ui_asset: UiAssetEditorPaneData,
    pub animation: AnimationEditorPaneData,
}

impl PaneData {
    pub(crate) fn template_nodes(&self) -> Option<&ModelRc<TemplatePaneNodeData>> {
        if self.template_v2.nodes.row_count() > 0 {
            return Some(&self.template_v2.nodes);
        }

        match self.kind.as_str() {
            "Hierarchy" => Some(&self.hierarchy.nodes),
            "Inspector" => Some(&self.inspector.nodes),
            "Console" => Some(&self.console.nodes),
            "Assets" => Some(&self.assets_activity.nodes),
            "AssetBrowser" => Some(&self.asset_browser.nodes),
            "Welcome" => Some(&self.welcome.nodes),
            "Project" | "UiComponentShowcase" => Some(&self.project_overview.nodes),
            "RuntimeDiagnostics" => Some(&self.runtime_diagnostics.nodes),
            "PerformanceTimeline" => Some(&self.performance_timeline.nodes),
            "ModulePlugins" => Some(&self.module_plugins.nodes),
            "BuildExport" => Some(&self.build_export.nodes),
            "GeneratedBottom" => Some(&self.generated_bottom.nodes),
            "UiAssetEditor" => Some(&self.ui_asset.nodes),
            "AnimationSequenceEditor" | "AnimationGraphEditor" => Some(&self.animation.nodes),
            _ => None,
        }
    }
}
