use super::support::*;

#[test]
fn editor_ui_host_runtime_registers_projection_bindings_as_route_stubs() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();
    let mut projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();

    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();

    assert!(!projection.bindings.is_empty());
    for binding in &projection.bindings {
        let route_id = binding.route_id.expect("route id");
        assert_eq!(
            service.route_binding(route_id).unwrap(),
            binding.binding.as_ui_binding()
        );
    }
}
