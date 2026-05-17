use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};

use crate::ui::host::module::{self, module_descriptor, EDITOR_MANAGER_NAME};
use crate::ui::host::EditorManager;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::preset::EditorUiDesignStack;
use crate::ui::workbench::view::{
    PanePayloadKind, PaneRouteNamespace, PreferredHost, ViewDescriptorId, ViewHost, ViewKind,
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
        ("editor.asset_browser_window", "editor.window.asset"),
        ("editor.ui_asset", "editor.window.ui_layout_editor"),
        (
            "editor.ui_asset_editor_window",
            "editor.window.ui_layout_editor",
        ),
        (
            "editor.ui_component_showcase",
            "editor.window.ui_component_showcase",
        ),
        ("editor.material_demo_window", "editor.window.material_demo"),
        (
            "editor.material_component_lab",
            "editor.window.material_component_lab",
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

#[test]
fn builtin_activity_windows_cover_default_design_stack_windows() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let descriptors = manager.descriptors();
    let layout = EditorUiDesignStack::material_fyrox_jetbrains_unreal().default_workbench_layout();

    for window in layout.activity_windows.values() {
        let descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.descriptor_id == window.descriptor_id)
            .unwrap_or_else(|| {
                panic!(
                    "missing builtin descriptor for preset window `{}`",
                    window.descriptor_id.0
                )
            });
        assert_eq!(descriptor.kind, ViewKind::ActivityWindow);
    }
}

#[test]
fn builtin_view_descriptors_cover_default_design_stack_view_instances() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let descriptors = manager.descriptors();
    let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();

    for instance in stack.default_view_instances() {
        assert!(
            descriptors
                .iter()
                .any(|descriptor| descriptor.descriptor_id == instance.descriptor_id),
            "missing builtin descriptor for preset view instance `{}` / descriptor `{}`",
            instance.instance_id.0,
            instance.descriptor_id.0
        );
    }
}

#[test]
fn unreal_style_feature_window_descriptors_use_expected_hosts() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let descriptors = manager.descriptors();

    for descriptor_id in [
        "editor.prefab_editor_window",
        "editor.material_editor_window",
        "editor.ui_asset_editor_window",
        "editor.animation_editor_window",
    ] {
        let descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.descriptor_id == ViewDescriptorId::new(descriptor_id))
            .unwrap_or_else(|| panic!("missing feature window descriptor `{descriptor_id}`"));
        assert_eq!(descriptor.kind, ViewKind::ActivityWindow);
        assert!(descriptor.multi_instance);
        assert_eq!(descriptor.preferred_host, PreferredHost::FloatingWindow);
    }

    for descriptor_id in ["editor.asset_browser_window", "editor.diagnostics_window"] {
        let descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.descriptor_id == ViewDescriptorId::new(descriptor_id))
            .unwrap_or_else(|| panic!("missing drawer-backed window descriptor `{descriptor_id}`"));
        assert_eq!(descriptor.kind, ViewKind::ActivityWindow);
        assert_eq!(descriptor.preferred_host, PreferredHost::ExclusiveMainPage);
    }
}

#[test]
fn material_demo_window_descriptor_opens_as_document_center_demo() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let descriptors = manager.descriptors();
    let descriptor = descriptors
        .iter()
        .find(|descriptor| {
            descriptor.descriptor_id == ViewDescriptorId::new("editor.material_demo_window")
        })
        .expect("missing Material demo window descriptor");

    assert_eq!(descriptor.kind, ViewKind::ActivityWindow);
    assert_eq!(descriptor.default_title, "Material Demo Window");
    assert_eq!(descriptor.preferred_host, PreferredHost::DocumentCenter);
    assert_eq!(descriptor.icon_key, "material-demo");
    assert_eq!(
        descriptor
            .activity_window_template
            .as_ref()
            .map(|template| template.document_id.as_str()),
        Some("editor.window.material_demo")
    );

    let instance_id = manager
        .open_view(ViewDescriptorId::new("editor.material_demo_window"), None)
        .expect("Material demo window should open through the view registry");
    assert_eq!(
        instance_id,
        crate::ui::workbench::view::ViewInstanceId::new("editor.material_demo_window#1")
    );
    assert_eq!(
        manager
            .current_view_instances()
            .into_iter()
            .find(|instance| instance.instance_id == instance_id)
            .map(|instance| instance.host),
        Some(crate::ui::workbench::view::ViewHost::Document(
            crate::ui::workbench::layout::MainPageId::workbench(),
            vec![]
        ))
    );
}

#[test]
fn component_showcase_window_descriptor_opens_as_exclusive_demo_page() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let descriptors = manager.descriptors();
    let descriptor = descriptors
        .iter()
        .find(|descriptor| {
            descriptor.descriptor_id == ViewDescriptorId::new("editor.ui_component_showcase")
        })
        .expect("missing UI Component Showcase descriptor");

    assert_eq!(descriptor.kind, ViewKind::ActivityWindow);
    assert_eq!(descriptor.default_title, "UI Component Showcase");
    assert_eq!(descriptor.preferred_host, PreferredHost::ExclusiveMainPage);
    assert_eq!(descriptor.icon_key, "ui-components");

    let instance_id = manager
        .open_view(ViewDescriptorId::new("editor.ui_component_showcase"), None)
        .expect("UI Component Showcase should open through the view registry");
    let page_id = MainPageId::new("page:editor.ui_component_showcase#1");
    assert_eq!(
        manager
            .current_view_instances()
            .into_iter()
            .find(|instance| instance.instance_id == instance_id)
            .map(|instance| instance.host),
        Some(ViewHost::ExclusivePage(page_id.clone()))
    );
    assert_eq!(manager.current_layout().active_main_page, page_id);
}

#[test]
fn functional_editor_internal_view_descriptors_use_document_host() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let descriptors = manager.descriptors();

    for descriptor_id in [
        "editor.prefab.viewport",
        "editor.prefab.inspector",
        "editor.material.graph",
        "editor.material.preview",
        "editor.ui.designer",
        "editor.ui.source",
        "editor.animation.timeline",
        "editor.animation.graph",
    ] {
        let descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.descriptor_id == ViewDescriptorId::new(descriptor_id))
            .unwrap_or_else(|| panic!("missing internal view descriptor `{descriptor_id}`"));
        assert_eq!(descriptor.kind, ViewKind::ActivityView);
        assert_eq!(descriptor.preferred_host, PreferredHost::DocumentCenter);
    }
}

#[test]
fn material_component_lab_window_descriptor_opens_as_exclusive_demo_page() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let descriptors = manager.descriptors();
    let descriptor = descriptors
        .iter()
        .find(|descriptor| {
            descriptor.descriptor_id == ViewDescriptorId::new("editor.material_component_lab")
        })
        .expect("missing Material Component Lab descriptor");

    assert_eq!(descriptor.kind, ViewKind::ActivityWindow);
    assert_eq!(descriptor.default_title, "Material Component Lab");
    assert_eq!(descriptor.preferred_host, PreferredHost::ExclusiveMainPage);
    assert_eq!(descriptor.icon_key, "material-component-lab");
    assert_eq!(
        descriptor
            .activity_window_template
            .as_ref()
            .map(|template| template.document_id.as_str()),
        Some("editor.window.material_component_lab")
    );

    let instance_id = manager
        .open_view(ViewDescriptorId::new("editor.material_component_lab"), None)
        .expect("Material Component Lab should open through the view registry");
    let page_id = MainPageId::new("page:editor.material_component_lab#1");
    assert_eq!(
        manager
            .current_view_instances()
            .into_iter()
            .find(|instance| instance.instance_id == instance_id)
            .map(|instance| instance.host),
        Some(ViewHost::ExclusivePage(page_id.clone()))
    );
    assert_eq!(manager.current_layout().active_main_page, page_id);
}

#[test]
fn debug_observatory_activity_window_reuses_runtime_diagnostics_payload() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let descriptors = manager.descriptors();

    let descriptor = descriptors
        .iter()
        .find(|descriptor| {
            descriptor.descriptor_id == ViewDescriptorId::new("editor.debug_observatory")
        })
        .expect("missing Debug Observatory descriptor");

    assert_eq!(descriptor.kind, ViewKind::ActivityWindow);
    assert_eq!(descriptor.default_title, "Debug Observatory");
    assert_eq!(descriptor.icon_key, "debug-observatory");
    assert!(descriptor
        .required_capabilities
        .contains(&crate::ui::host::EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS.to_string()));
    let pane_template = descriptor
        .pane_template
        .as_ref()
        .expect("Debug Observatory should reuse the Runtime Diagnostics pane template");
    assert_eq!(
        pane_template.body.document_id,
        "pane.runtime.diagnostics.body"
    );
    assert_eq!(
        pane_template.body.payload_kind,
        PanePayloadKind::RuntimeDiagnosticsV1
    );
    assert_eq!(
        pane_template.body.route_namespace,
        PaneRouteNamespace::Diagnostics
    );
}

#[test]
fn performance_timeline_activity_view_uses_dedicated_payload_and_diagnostics_capability() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let descriptors = manager.descriptors();

    let descriptor = descriptors
        .iter()
        .find(|descriptor| {
            descriptor.descriptor_id == ViewDescriptorId::new("editor.performance_timeline")
        })
        .expect("missing Performance Timeline descriptor");

    assert_eq!(descriptor.kind, ViewKind::ActivityView);
    assert_eq!(descriptor.default_title, "Performance Timeline");
    assert_eq!(descriptor.icon_key, "performance-timeline");
    assert!(descriptor
        .required_capabilities
        .contains(&crate::ui::host::EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS.to_string()));
    let pane_template = descriptor
        .pane_template
        .as_ref()
        .expect("Performance Timeline should own a pane template");
    assert_eq!(
        pane_template.body.document_id,
        "pane.performance.timeline.body"
    );
    assert_eq!(
        pane_template.body.payload_kind,
        PanePayloadKind::PerformanceTimelineV1
    );
    assert_eq!(
        pane_template.body.route_namespace,
        PaneRouteNamespace::Diagnostics
    );
}
