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
        self.panels.iter().find(|panel| panel.view_id == view_id)
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
        ),
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
mod tests {
    use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
    use zircon_runtime_interface::ui::skin::{
        FYROX_PANEL_PRESET_ID, JETBRAINS_SHELL_PRESET_ID, MATERIAL_DARK_SKIN_ID,
        UNREAL_WINDOW_MODEL_PRESET_ID,
    };

    use super::*;

    #[test]
    fn default_stack_binds_material_fyrox_jetbrains_and_unreal_roles() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();

        assert_eq!(stack.skin_id, MATERIAL_DARK_SKIN_ID);
        assert_eq!(stack.panel_preset_id, FYROX_PANEL_PRESET_ID);
        assert_eq!(stack.shell_preset_id, JETBRAINS_SHELL_PRESET_ID);
        assert_eq!(stack.window_model_preset_id, UNREAL_WINDOW_MODEL_PRESET_ID);
        assert!(!stack.shell.drawers.is_empty());
        assert_eq!(stack.window_model.windows.len(), 8);
        assert!(!stack.panels.is_empty());
    }

    #[test]
    fn default_stack_binds_unreal_window_model_contract() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();

        assert_eq!(
            stack.window_model.workbench().map(|window| window.kind),
            Some(EditorFunctionalWindowKind::Workbench)
        );
        assert_eq!(stack.window_model.feature_editor_windows().count(), 4);
        assert_eq!(stack.window_model.drawer_backed_windows().count(), 2);

        for window in stack.window_model.feature_editor_windows() {
            assert_eq!(window.dock_policy, EditorWindowDockPolicy::FloatingAllowed);
            assert!(!window.primary_views.is_empty());
        }
    }

    #[test]
    fn default_stack_models_feature_editors_as_independent_windows() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();

        for kind in [
            EditorFunctionalWindowKind::PrefabEditor,
            EditorFunctionalWindowKind::MaterialEditor,
            EditorFunctionalWindowKind::UiAssetEditor,
            EditorFunctionalWindowKind::AnimationEditor,
        ] {
            let window = stack.window(kind).expect("feature editor window");
            assert_eq!(window.dock_policy, EditorWindowDockPolicy::FloatingAllowed);
            assert!(!window.primary_views.is_empty());
            assert!(window
                .drawer_views
                .iter()
                .any(|view| view == "editor.inspector"));
        }

        let workbench = stack
            .window(EditorFunctionalWindowKind::Workbench)
            .expect("workbench window");
        assert_eq!(workbench.dock_policy, EditorWindowDockPolicy::MainWorkbench);
        assert!(workbench
            .drawer_views
            .iter()
            .any(|view| view == "editor.hierarchy"));
    }

    #[test]
    fn default_stack_binds_fyrox_panel_component_contracts() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();

        let hierarchy = stack.panel("editor.hierarchy").expect("hierarchy panel");
        assert_eq!(hierarchy.title, "Hierarchy");
        assert!(hierarchy
            .components
            .contains(&FyroxPanelComponentRole::TreeView));
        assert!(hierarchy
            .components
            .contains(&FyroxPanelComponentRole::SearchField));
        assert!(hierarchy
            .interactions
            .contains(&FyroxPanelInteraction::SelectionSync));

        let inspector = stack.panel("editor.inspector").expect("inspector panel");
        assert!(inspector
            .components
            .contains(&FyroxPanelComponentRole::PropertyGrid));
        assert!(inspector
            .components
            .contains(&FyroxPanelComponentRole::InspectorSection));
        assert!(inspector
            .components
            .contains(&FyroxPanelComponentRole::FieldEditor));

        let assets = stack.panel("editor.assets").expect("asset browser panel");
        assert_eq!(assets.title, "Asset Browser");
        assert!(assets
            .components
            .contains(&FyroxPanelComponentRole::FolderTree));
        assert!(assets
            .components
            .contains(&FyroxPanelComponentRole::AssetGrid));
        assert!(assets
            .components
            .contains(&FyroxPanelComponentRole::PreviewPane));

        let console = stack.panel("editor.console").expect("console panel");
        assert!(console
            .components
            .contains(&FyroxPanelComponentRole::VirtualList));
        assert!(console
            .components
            .contains(&FyroxPanelComponentRole::SeverityChips));
    }

    #[test]
    fn default_stack_has_panel_contracts_for_every_declared_view() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();

        for window in &stack.window_model.windows {
            for view_id in window
                .primary_views
                .iter()
                .chain(window.drawer_views.iter())
            {
                assert!(
                    stack.panel(view_id).is_some(),
                    "missing Fyrox panel preset for `{view_id}`"
                );
            }
        }
    }

    #[test]
    fn default_stack_fyrox_panel_roles_resolve_to_material_components() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
        let registry = UiComponentDescriptorRegistry::material_editor_foundation();

        for panel in &stack.panels {
            for role in &panel.components {
                let component_id = role.component_id();
                assert!(
                    registry.contains(component_id),
                    "panel `{}` role `{role:?}` resolves to missing component `{component_id}`",
                    panel.view_id
                );
            }
        }
    }

    #[test]
    fn default_stack_binds_jetbrains_shell_drawers_and_detach_contracts() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();

        let left_top = stack
            .shell
            .drawer(ActivityDrawerSlot::LeftTop)
            .expect("left top drawer");
        assert_eq!(
            left_top.visible_views,
            vec!["editor.hierarchy".to_string(), "editor.assets".to_string()]
        );
        assert_eq!(left_top.default_mode, ActivityDrawerMode::Pinned);
        assert!(left_top.allows_detach);
        assert!(left_top.allows_attach);
        assert!(left_top.collapse_to_activity_bar);
        assert!(left_top.persist_extent);

        let left_bottom = stack
            .shell
            .drawer(ActivityDrawerSlot::LeftBottom)
            .expect("left bottom drawer");
        assert_eq!(left_bottom.default_mode, ActivityDrawerMode::Collapsed);
        assert_eq!(
            left_bottom.visible_views,
            vec!["editor.module_plugins".to_string()]
        );

        let bottom = stack
            .shell
            .drawer(ActivityDrawerSlot::Bottom)
            .expect("bottom drawer");
        assert!(bottom
            .visible_views
            .iter()
            .any(|view| view == "editor.console"));
        assert!(bottom
            .visible_views
            .iter()
            .any(|view| view == "editor.runtime_diagnostics"));
        assert!(bottom
            .visible_views
            .iter()
            .any(|view| view == "editor.build_export_desktop"));

        assert!(stack.shell.tab_behavior.reorder_tabs);
        assert!(stack.shell.tab_behavior.activate_on_drop);
        assert!(stack.shell.floating_window_behavior.detach_to_native_window);
        assert!(
            stack
                .shell
                .floating_window_behavior
                .attach_to_original_drawer
        );
        assert!(
            stack
                .shell
                .floating_window_behavior
                .restore_hidden_drawers_on_attach
        );
    }

    #[test]
    fn default_stack_shell_contract_covers_workbench_drawer_views() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
        let shell_views = stack
            .shell
            .drawers
            .iter()
            .flat_map(|drawer| drawer.visible_views.iter())
            .collect::<std::collections::BTreeSet<_>>();
        let workbench = stack
            .window(EditorFunctionalWindowKind::Workbench)
            .expect("workbench window");

        for view in &workbench.drawer_views {
            assert!(
                shell_views.contains(view),
                "missing JetBrains shell drawer contract for `{view}`"
            );
        }
    }
}
