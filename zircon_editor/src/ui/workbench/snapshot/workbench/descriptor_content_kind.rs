use crate::ui::workbench::view::ViewDescriptorId;

use super::ViewContentKind;

pub(super) fn descriptor_content_kind(descriptor_id: &ViewDescriptorId) -> ViewContentKind {
    match descriptor_id.0.as_str() {
        "editor.welcome" => ViewContentKind::Welcome,
        "editor.project" => ViewContentKind::Project,
        "editor.hierarchy" => ViewContentKind::Hierarchy,
        "editor.inspector" => ViewContentKind::Inspector,
        "editor.scene" => ViewContentKind::Scene,
        "editor.game" => ViewContentKind::Game,
        "editor.assets" => ViewContentKind::Assets,
        "editor.console" => ViewContentKind::Console,
        "editor.prefab" => ViewContentKind::PrefabEditor,
        "editor.asset_browser" => ViewContentKind::AssetBrowser,
        "editor.ui_asset" => ViewContentKind::UiAssetEditor,
        "editor.ui_component_showcase" => ViewContentKind::UiComponentShowcase,
        "editor.animation_sequence" => ViewContentKind::AnimationSequenceEditor,
        "editor.animation_graph" => ViewContentKind::AnimationGraphEditor,
        "editor.runtime_diagnostics" => ViewContentKind::RuntimeDiagnostics,
        "editor.module_plugins" => ViewContentKind::ModulePlugins,
        _ => ViewContentKind::Placeholder,
    }
}
