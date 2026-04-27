use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};

use crate::ui::host::module::{self, module_descriptor, EDITOR_MANAGER_NAME};
use crate::ui::host::EditorManager;
use crate::ui::workbench::view::{ViewDescriptorId, ViewKind};

fn editor_runtime() -> CoreRuntime {
    let runtime = CoreRuntime::new();
    runtime.store_config_value(
        crate::ui::host::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
        serde_json::json!([
            crate::ui::host::EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
            crate::ui::host::EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING,
            crate::ui::host::EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
            crate::ui::host::EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING,
        ]),
    );
    runtime
        .register_module(foundation_module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime.activate_module(module::EDITOR_MODULE_NAME).unwrap();
    runtime
}

#[test]
fn builtin_activity_windows_expose_window_template_documents() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let descriptors = manager.descriptors();

    for (descriptor_id, template_document_id) in [
        ("editor.workbench_window", "editor.window.workbench"),
        ("editor.asset_browser", "editor.window.asset"),
        ("editor.ui_asset", "editor.window.ui_layout_editor"),
        (
            "editor.ui_component_showcase",
            "editor.window.ui_component_showcase",
        ),
    ] {
        let descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.descriptor_id == ViewDescriptorId::new(descriptor_id))
            .unwrap_or_else(|| panic!("missing builtin descriptor `{descriptor_id}`"));

        assert_eq!(descriptor.kind, ViewKind::ActivityWindow);
        assert_eq!(
            descriptor
                .activity_window_template
                .as_ref()
                .map(|template| template.document_id.as_str()),
            Some(template_document_id),
            "descriptor `{descriptor_id}` should use `{template_document_id}`"
        );
    }
}
