use crate::ui::template_runtime::EditorUiHostRuntime;
use std::path::PathBuf;

#[test]
fn editor_repository_host_window_template_file_loads_and_instantiates() {
    let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("ui")
        .join("editor")
        .join("host")
        .join("workbench_shell.v2.ui.toml");
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    runtime
        .register_document_file("ui.host_window.file", template_path)
        .unwrap();

    let projection = runtime.project_document("ui.host_window.file").unwrap();
    assert_eq!(projection.root.component, "UiHostWindow");
    assert_eq!(projection.root.children.len(), 4);
    assert_eq!(projection.root.children[0].component, "VerticalGroup");
    assert_eq!(projection.root.children[1].component, "Overlay");
    assert_eq!(projection.root.children[2].component, "Overlay");
    assert_eq!(projection.root.children[3].component, "Overlay");
}
