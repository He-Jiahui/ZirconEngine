use crate::ui::template::EditorTemplateRegistry;
use std::path::PathBuf;
use zircon_runtime::ui::template::UiAssetLoader;

#[test]
fn editor_repository_host_window_template_file_loads_and_instantiates() {
    let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("ui")
        .join("editor")
        .join("host")
        .join("workbench_shell.ui.toml");
    let document = UiAssetLoader::load_toml_file(&template_path).unwrap();

    let mut registry = EditorTemplateRegistry::default();
    registry
        .register_asset_document("ui.host_window.file", document)
        .unwrap();

    let instance = registry.instantiate("ui.host_window.file").unwrap();
    assert_eq!(instance.root.component.as_deref(), Some("UiHostWindow"));
    assert_eq!(instance.root.children.len(), 5);
    assert_eq!(
        instance.root.children[0].component.as_deref(),
        Some("VerticalBox")
    );
    assert_eq!(
        instance.root.children[1].component.as_deref(),
        Some("Container")
    );
    assert_eq!(
        instance.root.children[2].component.as_deref(),
        Some("Overlay")
    );
    assert_eq!(
        instance.root.children[3].component.as_deref(),
        Some("Overlay")
    );
    assert_eq!(
        instance.root.children[4].component.as_deref(),
        Some("Overlay")
    );
    assert_eq!(instance.root.children[0].children.len(), 3);
    assert_eq!(
        instance.root.children[0].children[0].component.as_deref(),
        Some("UiHostToolbar")
    );
    assert_eq!(
        instance.root.children[0].children[1].component.as_deref(),
        Some("HorizontalBox")
    );
    assert_eq!(
        instance.root.children[0].children[2].component.as_deref(),
        Some("StatusBar")
    );
    assert_eq!(instance.root.children[0].children[1].children.len(), 2);
    assert_eq!(
        instance.root.children[0].children[1].children[0]
            .component
            .as_deref(),
        Some("ActivityRail")
    );
    assert_eq!(
        instance.root.children[0].children[1].children[1]
            .component
            .as_deref(),
        Some("DocumentHost")
    );
}
