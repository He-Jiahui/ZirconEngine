use super::support::*;
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::template_runtime::EditorUiHostRuntime;
use crate::ui::workbench::view::ViewDescriptorId;

#[test]
fn builtin_pane_body_documents_match_descriptor_ids_and_runtime_registration() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let descriptors = manager.descriptors();

    let mut ui_runtime = EditorUiHostRuntime::default();
    ui_runtime.load_builtin_host_templates().unwrap();

    let cases = [
        (
            "editor.console",
            "res://ui/editor/host/console_body.zui",
            "ConsolePaneBody",
            Some("ConsolePaneBody/ClearConsole"),
        ),
        (
            "editor.inspector",
            "res://ui/editor/host/inspector_body.zui",
            "InspectorPaneBody",
            None,
        ),
        (
            "editor.hierarchy",
            "res://ui/editor/host/hierarchy_body.zui",
            "HierarchyPaneBody",
            Some("HierarchyPaneBody/SelectRoot"),
        ),
        (
            "editor.animation_sequence",
            "res://ui/editor/host/animation_sequence_body.zui",
            "AnimationSequencePaneBody",
            None,
        ),
        (
            "editor.animation_graph",
            "res://ui/editor/host/animation_graph_body.zui",
            "AnimationGraphPaneBody",
            None,
        ),
        (
            "editor.runtime_diagnostics",
            "res://ui/editor/host/runtime_diagnostics_body.zui",
            "RuntimeDiagnosticsPaneBody",
            Some("RuntimeDiagnosticsPaneBody/FocusDiagnostics"),
        ),
        (
            "editor.performance_timeline",
            "res://ui/editor/host/performance_timeline_body.zui",
            "PerformanceTimelinePaneBody",
            None,
        ),
        (
            "editor.module_plugins",
            "res://ui/editor/host/module_plugins_body.zui",
            "ModulePluginsPaneBody",
            Some("ModulePluginsPaneBody/FocusModulePlugins"),
        ),
        (
            "editor.build_export_desktop",
            "res://ui/editor/host/build_export_desktop_body.zui",
            "BuildExportPaneBody",
            Some("BuildExportPaneBody/FocusBuildExport"),
        ),
        (
            "editor.generated_bottom",
            "res://ui/editor/host/generated_bottom_body.zui",
            "GeneratedBottomPaneBody",
            Some("WorkbenchGeneratedBottom/OpenPanel"),
        ),
    ];

    for (descriptor_id, document_id, component_id, binding_id) in cases {
        let descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.descriptor_id == ViewDescriptorId::new(descriptor_id))
            .unwrap_or_else(|| panic!("missing builtin descriptor `{descriptor_id}`"));
        let pane_template = descriptor
            .pane_template
            .as_ref()
            .unwrap_or_else(|| panic!("descriptor `{descriptor_id}` is missing pane_template"));

        assert_eq!(
            pane_template.body.document_id, document_id,
            "descriptor `{descriptor_id}` must use the stable pane body document id"
        );

        let component = ui_runtime
            .component_descriptor(component_id)
            .unwrap_or_else(|| panic!("missing builtin component descriptor `{component_id}`"));
        assert_eq!(component.document_id, document_id);
        assert_eq!(component.binding_namespace, component_id);

        let projection = ui_runtime
            .project_document(document_id)
            .unwrap_or_else(|error| {
                panic!("failed to project builtin pane body document `{document_id}`: {error}")
            });
        assert_eq!(projection.document_id, document_id);
        assert!(
            matches!(
                projection.root.component.as_str(),
                "VerticalBox" | "VerticalGroup"
            ),
            "document `{document_id}` should project a vertical root layout, got `{}`",
            projection.root.component
        );
        if let Some(binding_id) = binding_id {
            assert!(
                projection
                    .bindings
                    .iter()
                    .any(|binding| binding.binding_id == binding_id),
                "document `{document_id}` must expose binding `{binding_id}` through runtime projection"
            );
        }
    }
}
