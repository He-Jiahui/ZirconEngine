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
        ActivityDrawerSlot::BottomLeft => "bottom_left",
        ActivityDrawerSlot::BottomRight => "bottom_right",
    }
}

pub(super) fn menu_id(label: &str) -> String {
    label.to_ascii_lowercase().replace(' ', "_")
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
        ViewContentKind::AnimationSequenceEditor => "animation_sequence_editor",
        ViewContentKind::AnimationGraphEditor => "animation_graph_editor",
        ViewContentKind::Placeholder => "placeholder",
    }
}
