use super::super::support::*;
use super::support::surface_control_frame;

#[test]
fn builtin_host_drawer_source_document_can_be_loaded_as_shared_surface() {
    let _guard = env_lock().lock().unwrap();

    let mut runtime = crate::ui::template_runtime::EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    let mut surface = runtime
        .build_shared_surface("workbench.drawer_source")
        .expect("drawer source document should be registered as a builtin shared surface");
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();

    assert_eq!(
        surface_control_frame(&surface, "WorkbenchDrawerTopBarRoot"),
        Some(UiFrame::new(0.0, 0.0, 1280.0, 59.0))
    );
    assert_eq!(
        surface_control_frame(&surface, "WorkbenchDrawerStatusBarRoot"),
        Some(UiFrame::new(0.0, 696.0, 1280.0, 24.0))
    );
}
