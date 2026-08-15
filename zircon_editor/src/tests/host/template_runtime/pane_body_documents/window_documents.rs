use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::template_runtime::EditorUiHostRuntime;

#[test]
fn builtin_activity_window_documents_are_registered_in_host_runtime() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut ui_runtime = EditorUiHostRuntime::default();
    ui_runtime.load_builtin_host_templates().unwrap();

    for document_id in [
        "res://ui/editor/host/editor_main_frame.zui",
        "res://ui/editor/windows/workbench_window.zui",
        "res://ui/editor/windows/asset_window.zui",
        "res://ui/editor/windows/ui_layout_editor_window.zui",
        "res://ui/editor/component_showcase.zui",
        "res://ui/editor/material_demo_window.zui",
    ] {
        let projection = ui_runtime
            .project_document(document_id)
            .unwrap_or_else(|error| panic!("failed to project `{document_id}`: {error}"));
        assert_eq!(projection.document_id, document_id);
        assert!(
            matches!(
                projection.root.component.as_str(),
                "Overlay" | "VerticalBox" | "VerticalGroup" | "WindowFrame"
            ),
            "`{document_id}` should project a supported root layout, got `{}`",
            projection.root.component
        );
    }
}

#[test]
fn material_demo_window_document_projects_material_primitives() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut ui_runtime = EditorUiHostRuntime::default();
    ui_runtime.load_builtin_host_templates().unwrap();

    let document_id = "res://ui/editor/material_demo_window.zui";
    let projection = ui_runtime.project_document(document_id).unwrap();
    let mut surface = ui_runtime.build_shared_surface(document_id).unwrap();
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();
    let host_projection = ui_runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap();

    assert_eq!(projection.root.component, "WindowFrame");
    for control_id in [
        "PrimaryButton",
        "IconButton",
        "TextField",
        "Checkbox",
        "Switch",
        "Dropdown",
        "Slider",
        "Tabs",
        "Scrollbar",
        "Splitter",
        "Modal",
    ] {
        assert!(
            host_projection.node_by_control_id(control_id).is_some(),
            "Material demo should project primitive control `{control_id}`"
        );
    }
}
