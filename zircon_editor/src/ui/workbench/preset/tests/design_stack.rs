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
