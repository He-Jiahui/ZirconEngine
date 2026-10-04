use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::skin::{
    FYROX_PANEL_PRESET_ID, JETBRAINS_SHELL_PRESET_ID, MATERIAL_DARK_SKIN_ID,
    UNREAL_WINDOW_MODEL_PRESET_ID,
};

use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};

use super::functional_window::{
    EditorFunctionalWindowKind, EditorFunctionalWindowPreset, EditorWindowDockPolicy,
    UnrealWindowModelPreset,
};
use super::panel_preset::{FyroxPanelComponentRole, FyroxPanelInteraction, FyroxPanelPreset};
use super::shell_preset::{JetBrainsDrawerPreset, JetBrainsShellPreset};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorUiDesignStack {
    pub skin_id: String,
    pub panel_preset_id: String,
    pub shell_preset_id: String,
    pub window_model_preset_id: String,
    pub shell: JetBrainsShellPreset,
    pub panels: Vec<FyroxPanelPreset>,
    pub window_model: UnrealWindowModelPreset,
}

impl EditorUiDesignStack {
    pub fn material_fyrox_jetbrains_unreal() -> Self {
        Self {
            skin_id: MATERIAL_DARK_SKIN_ID.to_string(),
            panel_preset_id: FYROX_PANEL_PRESET_ID.to_string(),
            shell_preset_id: JETBRAINS_SHELL_PRESET_ID.to_string(),
            window_model_preset_id: UNREAL_WINDOW_MODEL_PRESET_ID.to_string(),
            shell: default_jetbrains_shell_preset(),
            panels: default_fyrox_panel_presets(),
            window_model: UnrealWindowModelPreset::new(default_functional_windows()),
        }
    }

    pub fn window(
        &self,
        kind: EditorFunctionalWindowKind,
    ) -> Option<&EditorFunctionalWindowPreset> {
        self.window_model.window(kind)
    }

    pub fn panel(&self, view_id: &str) -> Option<&FyroxPanelPreset> {
        panel_by_default_index(&self.panels, view_id)
    }
}

fn panel_by_default_index<'a>(
    panels: &'a [FyroxPanelPreset],
    view_id: &str,
) -> Option<&'a FyroxPanelPreset> {
    default_panel_index(view_id)
        .and_then(|expected_index| panels.get(expected_index))
        .filter(|panel| panel.view_id == view_id)
        .or_else(|| panels.iter().find(|panel| panel.view_id == view_id))
}

const fn default_panel_index(view_id: &str) -> Option<usize> {
    match view_id.as_bytes() {
        b"editor.scene" => Some(0),
        b"editor.game" => Some(1),
        b"editor.hierarchy" => Some(2),
        b"editor.inspector" => Some(3),
        b"editor.assets" => Some(4),
        b"editor.console" => Some(5),
        b"editor.runtime_diagnostics" => Some(6),
        b"editor.build_export_desktop" => Some(7),
        b"editor.module_plugins" => Some(8),
        b"editor.prefab.viewport" => Some(9),
        b"editor.prefab.inspector" => Some(10),
        b"editor.material.graph" => Some(11),
        b"editor.material.preview" => Some(12),
        b"editor.ui.designer" => Some(13),
        b"editor.ui.source" => Some(14),
        b"editor.animation.timeline" => Some(15),
        b"editor.animation.graph" => Some(16),
        b"editor.asset_browser" => Some(17),
        b"editor.asset_preview" => Some(18),
        b"editor.asset_metadata" => Some(19),
        _ => None,
    }
}

fn default_jetbrains_shell_preset() -> JetBrainsShellPreset {
    JetBrainsShellPreset::new([
        JetBrainsDrawerPreset::new(
            ActivityDrawerSlot::LeftTop,
            "Project Tools",
            ["editor.hierarchy", "editor.assets"],
        ),
        JetBrainsDrawerPreset::new(
            ActivityDrawerSlot::LeftBottom,
            "Modules",
            ["editor.module_plugins"],
        )
        .with_default_mode(ActivityDrawerMode::Collapsed),
        JetBrainsDrawerPreset::new(
            ActivityDrawerSlot::RightTop,
            "Inspector",
            ["editor.inspector"],
        ),
        JetBrainsDrawerPreset::new(
            ActivityDrawerSlot::Bottom,
            "Output",
            [
                "editor.console",
                "editor.runtime_diagnostics",
                "editor.build_export_desktop",
            ],
        )
        .with_default_mode(ActivityDrawerMode::Collapsed),
    ])
}

fn default_fyrox_panel_presets() -> Vec<FyroxPanelPreset> {
    use FyroxPanelComponentRole::{
        AssetGrid, AssetList, CategorizedList, ContextMenu, FieldEditor, FilterBar, FolderTree,
        GizmoControls, GraphCanvas, InspectorSection, MetadataPane, PaneToolbar, PreviewPane,
        PropertyGrid, SearchField, SeverityChips, SourceEditor, StatusActionControls, Timeline,
        TreeView, ViewportHost, VirtualList, VisualDesigner,
    };
    use FyroxPanelInteraction::{
        AssetPreview, ContextMenu as ContextMenuInteraction, DataRefresh, DetachAttach,
        MetadataEdit, PluginEnableDisable, PropertyEdit, SceneGizmo, SearchFilter, SelectionSync,
        SeverityFilter, SourceEdit, TimelineScrub, VirtualizedScroll,
    };

    vec![
        FyroxPanelPreset::new("editor.scene", "Scene")
            .with_components([ViewportHost, PaneToolbar, GizmoControls])
            .with_interactions([SceneGizmo, SelectionSync, DetachAttach]),
        FyroxPanelPreset::new("editor.game", "Game")
            .with_components([ViewportHost, PaneToolbar])
            .with_interactions([DataRefresh, DetachAttach]),
        FyroxPanelPreset::new("editor.hierarchy", "Hierarchy")
            .with_components([SearchField, TreeView, ContextMenu])
            .with_interactions([SearchFilter, SelectionSync, ContextMenuInteraction]),
        FyroxPanelPreset::new("editor.inspector", "Inspector")
            .with_components([PropertyGrid, InspectorSection, FieldEditor])
            .with_interactions([PropertyEdit, SelectionSync, DataRefresh]),
        FyroxPanelPreset::new("editor.assets", "Asset Browser")
            .with_components([
                SearchField,
                FolderTree,
                AssetGrid,
                AssetList,
                PreviewPane,
                MetadataPane,
            ])
            .with_interactions([
                SearchFilter,
                AssetPreview,
                MetadataEdit,
                ContextMenuInteraction,
            ]),
        FyroxPanelPreset::new("editor.console", "Console")
            .with_components([FilterBar, VirtualList, SeverityChips])
            .with_interactions([SeverityFilter, VirtualizedScroll, DataRefresh]),
        FyroxPanelPreset::new("editor.runtime_diagnostics", "Runtime Diagnostics")
            .with_components([FilterBar, VirtualList, PropertyGrid])
            .with_interactions([SeverityFilter, VirtualizedScroll, DataRefresh]),
        FyroxPanelPreset::new("editor.build_export_desktop", "Desktop Export")
            .with_components([PropertyGrid, StatusActionControls])
            .with_interactions([DataRefresh, PropertyEdit]),
        FyroxPanelPreset::new("editor.module_plugins", "Plugin Manager")
            .with_components([SearchField, CategorizedList, StatusActionControls])
            .with_interactions([SearchFilter, PluginEnableDisable, DataRefresh]),
        FyroxPanelPreset::new("editor.prefab.viewport", "Prefab Viewport")
            .with_components([ViewportHost, PaneToolbar, GizmoControls])
            .with_interactions([SceneGizmo, SelectionSync, DetachAttach]),
        FyroxPanelPreset::new("editor.prefab.inspector", "Prefab Inspector")
            .with_components([PropertyGrid, InspectorSection, FieldEditor])
            .with_interactions([PropertyEdit, SelectionSync, DataRefresh]),
        FyroxPanelPreset::new("editor.material.graph", "Material Graph")
            .with_components([GraphCanvas, PaneToolbar, PreviewPane])
            .with_interactions([SelectionSync, PropertyEdit, DataRefresh]),
        FyroxPanelPreset::new("editor.material.preview", "Material Preview")
            .with_components([ViewportHost, PreviewPane, PropertyGrid])
            .with_interactions([AssetPreview, PropertyEdit, DataRefresh]),
        FyroxPanelPreset::new("editor.ui.designer", "UI Designer")
            .with_components([VisualDesigner, PaneToolbar, PropertyGrid])
            .with_interactions([SelectionSync, PropertyEdit, DetachAttach]),
        FyroxPanelPreset::new("editor.ui.source", "UI Source")
            .with_components([SourceEditor, PaneToolbar])
            .with_interactions([SourceEdit, DataRefresh]),
        FyroxPanelPreset::new("editor.animation.timeline", "Animation Timeline")
            .with_components([Timeline, PaneToolbar, PropertyGrid])
            .with_interactions([TimelineScrub, SelectionSync, PropertyEdit]),
        FyroxPanelPreset::new("editor.animation.graph", "Animation Graph")
            .with_components([GraphCanvas, PaneToolbar, PropertyGrid])
            .with_interactions([SelectionSync, PropertyEdit, DataRefresh]),
        FyroxPanelPreset::new("editor.asset_browser", "Asset Browser")
            .with_components([
                SearchField,
                FolderTree,
                AssetGrid,
                AssetList,
                PreviewPane,
                MetadataPane,
            ])
            .with_interactions([
                SearchFilter,
                AssetPreview,
                MetadataEdit,
                ContextMenuInteraction,
            ]),
        FyroxPanelPreset::new("editor.asset_preview", "Asset Preview")
            .with_components([PreviewPane, MetadataPane])
            .with_interactions([AssetPreview, MetadataEdit, DataRefresh]),
        FyroxPanelPreset::new("editor.asset_metadata", "Asset Metadata")
            .with_components([PropertyGrid, MetadataPane, FieldEditor])
            .with_interactions([MetadataEdit, PropertyEdit, DataRefresh]),
    ]
}

fn default_functional_windows() -> Vec<EditorFunctionalWindowPreset> {
    use EditorFunctionalWindowKind::{
        AnimationEditor, AssetBrowser, Diagnostics, MaterialEditor, PrefabEditor, SceneGame,
        UiAssetEditor, Workbench,
    };
    use EditorWindowDockPolicy::{DockedDocument, DrawerBacked, FloatingAllowed, MainWorkbench};

    vec![
        EditorFunctionalWindowPreset::new(Workbench, "Workbench", MainWorkbench)
            .with_primary_views(["editor.scene", "editor.game"])
            .with_drawer_views([
                "editor.hierarchy",
                "editor.inspector",
                "editor.assets",
                "editor.console",
                "editor.runtime_diagnostics",
                "editor.build_export_desktop",
                "editor.module_plugins",
            ]),
        EditorFunctionalWindowPreset::new(SceneGame, "Scene/Game", DockedDocument)
            .with_primary_views(["editor.scene", "editor.game"]),
        EditorFunctionalWindowPreset::new(PrefabEditor, "Prefab Editor", FloatingAllowed)
            .with_primary_views(["editor.prefab.viewport", "editor.prefab.inspector"])
            .with_drawer_views([
                "editor.hierarchy",
                "editor.inspector",
                "editor.asset_browser",
            ]),
        EditorFunctionalWindowPreset::new(MaterialEditor, "Material Editor", FloatingAllowed)
            .with_primary_views(["editor.material.graph", "editor.material.preview"])
            .with_drawer_views(["editor.inspector", "editor.asset_browser"]),
        EditorFunctionalWindowPreset::new(UiAssetEditor, "UI Asset Editor", FloatingAllowed)
            .with_primary_views(["editor.ui.designer", "editor.ui.source"])
            .with_drawer_views(["editor.inspector", "editor.asset_browser"]),
        EditorFunctionalWindowPreset::new(AnimationEditor, "Animation Editor", FloatingAllowed)
            .with_primary_views(["editor.animation.timeline", "editor.animation.graph"])
            .with_drawer_views(["editor.inspector", "editor.asset_browser"]),
        EditorFunctionalWindowPreset::new(AssetBrowser, "Asset Browser", DrawerBacked)
            .with_primary_views(["editor.asset_browser"])
            .with_drawer_views(["editor.asset_preview", "editor.asset_metadata"]),
        EditorFunctionalWindowPreset::new(Diagnostics, "Diagnostics", DrawerBacked)
            .with_primary_views(["editor.console", "editor.runtime_diagnostics"])
            .with_drawer_views(["editor.module_plugins"]),
    ]
}

#[cfg(test)]
#[path = "tests/design_stack.rs"]
mod tests;

#[cfg(test)]
#[path = "design_stack/tests/default_panel_index_tests.rs"]
mod default_panel_index_tests;
