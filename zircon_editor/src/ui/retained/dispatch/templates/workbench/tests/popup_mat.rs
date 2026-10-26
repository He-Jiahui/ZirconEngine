use super::*;
use crate::core::settings::{SettingsMutationCoordinator, EDITOR_LOCALE_KEY};
use crate::ui::workbench::reference::EditorWorkbenchTemplateSurfaceError;
use zircon_runtime_interface::ui::{layout::UiSize, tree::UiTreeError};

#[test]
fn popup_materialization_success_retains_the_original_displayed_context() {
    let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    bridge
        .open_settings_window(guarded_preferences_tests::open_state(
            &mutations,
            "builtin|settings.category.editor",
        ))
        .unwrap();
    let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
    let generation = mutations.authority().snapshot().generation();
    assert!(bridge
        .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
        .unwrap());
    assert_eq!(bridge.settings_values_context(), Some(&displayed));
    assert!(bridge.close_settings_editor().unwrap());
    assert_eq!(bridge.settings_values_context(), Some(&displayed));
    assert!(bridge
        .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
        .unwrap());
    assert!(bridge.update_settings_scroll_offsets(12.0, 24.0).unwrap());
    assert!(bridge
        .settings_editor_open_key(WorkbenchSettingsEditorKind::Enum)
        .is_none());
    assert_eq!(bridge.settings_values_context(), Some(&displayed));
    assert!(!bridge.update_settings_scroll_offsets(12.0, 24.0).unwrap());
    assert_eq!(bridge.settings_values_context(), Some(&displayed));
    assert!(!bridge
        .update_settings_scroll_offsets(f32::NAN, 24.0)
        .unwrap());
    assert_eq!(bridge.settings_values_context(), Some(&displayed));
    assert_eq!(mutations.authority().snapshot().generation(), generation);
    mutations.shutdown().wait();
}

#[test]
fn popup_materialization_failure_revokes_context_until_normal_value_refresh() {
    for transition in ["open", "close", "scroll"] {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                .unwrap();
        bridge
            .open_settings_window(guarded_preferences_tests::open_state(
                &mutations,
                "builtin|settings.category.editor",
            ))
            .unwrap();
        let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
        if transition != "open" {
            bridge
                .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
                .unwrap();
        }
        // Remove one actual authored node while keeping its parent edge. Force the normal
        // layout pass to observe that missing source node after the popup property writes.
        let missing = bridge.control_node_id("WorkbenchSceneTitle").unwrap();
        let removed = bridge
            .template_surface
            .surface
            .tree
            .nodes
            .remove(&missing)
            .unwrap();
        for root in bridge.template_surface.surface.tree.roots.clone() {
            let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
            node.dirty.layout = true;
            node.dirty.input = true;
        }
        let result = match transition {
            "open" => bridge
                .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum),
            "close" => bridge.close_settings_editor(),
            "scroll" => bridge.update_settings_scroll_offsets(12.0, 24.0),
            _ => unreachable!(),
        };
        let error = result.unwrap_err();
        assert!(
            matches!(
                &error,
                BuiltinHostWindowTemplateBridgeError::ComponentizedWorkbench(
                    EditorWorkbenchTemplateSurfaceError::Tree(UiTreeError::MissingNode(id))
                ) if *id == missing
            ),
            "{transition}: {error}"
        );
        assert!(bridge.settings_values_context().is_none());
        assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
        assert_eq!(
            mutations.authority().snapshot().generation(),
            displayed.generation()
        );
        bridge
            .template_surface
            .surface
            .tree
            .nodes
            .insert(missing, removed);
        bridge.refresh_settings_values(&displayed).unwrap();
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        mutations.shutdown().wait();
    }
}
