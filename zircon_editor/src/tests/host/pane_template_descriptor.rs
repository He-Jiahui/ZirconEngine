use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};

use crate::ui::host::module::{self, module_descriptor, EDITOR_MANAGER_NAME};
use crate::ui::host::EditorManager;
use crate::ui::workbench::view::{
    PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, ViewDescriptorId,
};

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
fn builtin_pane_views_expose_template_metadata() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let descriptors = manager.descriptors();

    let cases = [
        (
            "editor.console",
            PanePayloadKind::ConsoleV1,
            PaneRouteNamespace::Dock,
            PaneInteractionMode::TemplateOnly,
        ),
        (
            "editor.inspector",
            PanePayloadKind::InspectorV1,
            PaneRouteNamespace::Draft,
            PaneInteractionMode::TemplateOnly,
        ),
        (
            "editor.hierarchy",
            PanePayloadKind::HierarchyV1,
            PaneRouteNamespace::Selection,
            PaneInteractionMode::HybridNativeSlot,
        ),
        (
            "editor.animation_sequence",
            PanePayloadKind::AnimationSequenceV1,
            PaneRouteNamespace::Animation,
            PaneInteractionMode::HybridNativeSlot,
        ),
        (
            "editor.animation_graph",
            PanePayloadKind::AnimationGraphV1,
            PaneRouteNamespace::Animation,
            PaneInteractionMode::HybridNativeSlot,
        ),
        (
            "editor.runtime_diagnostics",
            PanePayloadKind::RuntimeDiagnosticsV1,
            PaneRouteNamespace::Diagnostics,
            PaneInteractionMode::TemplateOnly,
        ),
        (
            "editor.module_plugins",
            PanePayloadKind::ModulePluginsV1,
            PaneRouteNamespace::Dock,
            PaneInteractionMode::HybridNativeSlot,
        ),
        (
            "editor.ui_component_showcase",
            PanePayloadKind::UiComponentShowcaseV1,
            PaneRouteNamespace::UiComponentShowcase,
            PaneInteractionMode::TemplateOnly,
        ),
    ];

    for (descriptor_id, payload_kind, route_namespace, interaction_mode) in cases {
        let descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.descriptor_id == ViewDescriptorId::new(descriptor_id))
            .unwrap_or_else(|| panic!("missing builtin descriptor `{descriptor_id}`"));
        let pane_template = descriptor
            .pane_template
            .as_ref()
            .unwrap_or_else(|| panic!("descriptor `{descriptor_id}` is missing pane_template"));

        assert_eq!(pane_template.body.payload_kind, payload_kind);
        assert_eq!(pane_template.body.route_namespace, route_namespace);
        assert_eq!(pane_template.body.interaction_mode, interaction_mode);
        assert!(
            !pane_template.body.document_id.is_empty(),
            "descriptor `{descriptor_id}` must provide a body document id"
        );
    }
}
