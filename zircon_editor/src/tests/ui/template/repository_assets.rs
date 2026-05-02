use crate::ui::template::{EditorTemplateRegistry, EditorTemplateRuntimeService};
use std::path::PathBuf;

#[test]
fn editor_repository_host_window_template_file_loads_and_instantiates() {
    let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("ui")
        .join("editor")
        .join("host")
        .join("workbench_shell.ui.toml");
    let template_service = EditorTemplateRuntimeService;
    let document = template_service.load_document_file(&template_path).unwrap();
    let mut registry = EditorTemplateRegistry::default();
    template_service
        .register_asset_document(&mut registry, "ui.host_window.file", document)
        .unwrap();

    let instance = template_service
        .instantiate(&registry, "ui.host_window.file")
        .unwrap();
    assert_eq!(instance.root.component.as_deref(), Some("UiHostWindow"));
    assert_eq!(instance.root.children.len(), 4);
    assert_eq!(
        instance.root.children[0].component.as_deref(),
        Some("VerticalBox")
    );
    assert_eq!(
        instance.root.children[1].component.as_deref(),
        Some("Overlay")
    );
    assert_eq!(
        instance.root.children[2].component.as_deref(),
        Some("Overlay")
    );
    assert_eq!(
        instance.root.children[3].component.as_deref(),
        Some("Overlay")
    );
}
