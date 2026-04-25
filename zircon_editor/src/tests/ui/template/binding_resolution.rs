use super::support::EDITOR_HOST_WINDOW_ASSET_TOML;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::template::{EditorTemplateAdapter, EditorTemplateError, EditorTemplateRegistry};
use zircon_runtime::ui::binding::UiEventKind;

#[test]
fn editor_template_adapter_resolves_stable_binding_ids_to_typed_editor_bindings() {
    let document =
        crate::tests::support::load_test_ui_asset(EDITOR_HOST_WINDOW_ASSET_TOML).unwrap();
    let mut registry = EditorTemplateRegistry::default();
    registry
        .register_asset_document("ui.host_window", document)
        .unwrap();
    let instance = registry.instantiate("ui.host_window").unwrap();

    let mut adapter = EditorTemplateAdapter::default();
    adapter
        .register_binding(
            "WorkbenchMenuBar/OpenProject",
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "OpenProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("OpenProject"),
            ),
        )
        .unwrap();
    adapter
        .register_binding(
            "WorkbenchMenuBar/SaveProject",
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "SaveProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("SaveProject"),
            ),
        )
        .unwrap();

    let bindings = adapter.resolve_instance_bindings(&instance).unwrap();
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].path().event_kind, UiEventKind::Click);
    assert_eq!(bindings[0].path().view_id, "WorkbenchMenuBar");
    assert_eq!(bindings[0].path().control_id, "OpenProject");
    assert_eq!(bindings[1].path().control_id, "SaveProject");
}

#[test]
fn editor_template_adapter_rejects_missing_binding_resolution() {
    let document =
        crate::tests::support::load_test_ui_asset(EDITOR_HOST_WINDOW_ASSET_TOML).unwrap();
    let mut registry = EditorTemplateRegistry::default();
    registry
        .register_asset_document("ui.host_window", document)
        .unwrap();
    let instance = registry.instantiate("ui.host_window").unwrap();

    let error = EditorTemplateAdapter::default()
        .resolve_instance_bindings(&instance)
        .unwrap_err();
    assert_eq!(
        error,
        EditorTemplateError::MissingBinding {
            binding_id: "WorkbenchMenuBar/OpenProject".to_string(),
        }
    );
}
