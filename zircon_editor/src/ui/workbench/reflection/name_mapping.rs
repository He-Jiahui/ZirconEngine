use crate::ui::EditorActivityReflection;

use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::ViewContentKind;

pub(super) fn binding_view_id(activity: &EditorActivityReflection) -> String {
    match activity.descriptor_id.as_str() {
        "editor.project" => "ProjectView".to_string(),
        "editor.hierarchy" => "HierarchyView".to_string(),
        "editor.inspector" => "InspectorView".to_string(),
        "editor.scene" => "SceneView".to_string(),
        "editor.game" => "GameView".to_string(),
        "editor.assets" => "AssetsView".to_string(),
        "editor.console" => "ConsoleView".to_string(),
        "editor.runtime_diagnostics" => "RuntimeDiagnosticsView".to_string(),
        "editor.performance_timeline" => "PerformanceTimelineView".to_string(),
        "editor.debug_observatory" => "DebugObservatoryWindow".to_string(),
        "editor.build_export_desktop" => "BuildExportView".to_string(),
        "editor.generated_bottom" => "GeneratedBottomView".to_string(),
        "editor.prefab" => "PrefabEditorWindow".to_string(),
        "editor.asset_browser" => "AssetBrowserWindow".to_string(),
        _ => activity.instance_id.clone(),
    }
}

pub(super) fn drawer_slot_name(slot: ActivityDrawerSlot) -> &'static str {
    match slot {
        ActivityDrawerSlot::LeftTop => "left_top",
        ActivityDrawerSlot::LeftBottom => "left_bottom",
        ActivityDrawerSlot::RightTop => "right_top",
        ActivityDrawerSlot::RightBottom => "right_bottom",
        ActivityDrawerSlot::Bottom => "bottom",
    }
}

pub(super) fn menu_id(label: &str) -> String {
    let mut menu_id = String::with_capacity(label.len());
    menu_id.extend(label.chars().map(|character| {
        if character == ' ' {
            '_'
        } else {
            character.to_ascii_lowercase()
        }
    }));
    menu_id
}

pub(super) fn content_kind_name(kind: ViewContentKind) -> &'static str {
    match kind {
        ViewContentKind::Welcome => "welcome",
        ViewContentKind::Project => "project",
        ViewContentKind::Hierarchy => "hierarchy",
        ViewContentKind::Inspector => "inspector",
        ViewContentKind::Scene => "scene",
        ViewContentKind::Game => "game",
        ViewContentKind::Assets => "assets",
        ViewContentKind::Console => "console",
        ViewContentKind::PrefabEditor => "prefab_editor",
        ViewContentKind::AssetBrowser => "asset_browser",
        ViewContentKind::UiAssetEditor => "ui_asset_editor",
        ViewContentKind::UiComponentShowcase => "ui_component_showcase",
        ViewContentKind::AnimationSequenceEditor => "animation_sequence_editor",
        ViewContentKind::AnimationGraphEditor => "animation_graph_editor",
        ViewContentKind::RuntimeDiagnostics => "runtime_diagnostics",
        ViewContentKind::PerformanceTimeline => "performance_timeline",
        ViewContentKind::ModulePlugins => "module_plugins",
        ViewContentKind::BuildExport => "build_export",
        ViewContentKind::GeneratedBottom => "generated_bottom",
        ViewContentKind::Placeholder => "placeholder",
    }
}

#[cfg(test)]
#[path = "name_mapping/single_pass_tests.rs"]
mod single_pass_tests;
