use crate::ui::asset_editor::{
    UiAssetEditorCommand, UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession,
};
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

const VIEW: &str = r#"
[asset]
kind = "view"
id = "editor.test.close_dirty_preflight"
version = 2
display_name = "Before"

[root]
node = "root"

[nodes.root]
component = "VerticalGroup"
"#;

const COMPONENT: &str = r#"
[asset]
kind = "component"
id = "editor.test.close_dirty_preflight.component"
version = 2
display_name = "Imported Component"

[components.ImportedWidget]
root = "imported_root"

[nodes.imported_root]
component = "Text"
props = { text = "Imported" }
"#;

#[test]
fn failed_imported_widget_projection_keeps_command_source_unchanged() {
    let route = UiAssetEditorRoute::new(
        "editor.test.close_dirty_preflight",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_v2_source(route, VIEW, UiSize::new(640.0, 360.0))
        .expect("valid V2 view");
    let mut imported = UiV2AssetLoader::load_toml_str(COMPONENT).expect("valid V2 component");
    imported.components.get_mut("ImportedWidget").unwrap().root = "missing_root".into();
    session
        .v2_compiler_imports
        .widgets
        .insert("res://ui/widgets/imported.zui".to_string(), imported);
    let source_before = session.source_buffer().text().to_string();
    let revision_before = session.source_revision();

    assert!(session
        .apply_command(UiAssetEditorCommand::edit_source(
            VIEW.replace("Before", "After")
        ))
        .is_err());
    assert_eq!(session.source_buffer().text(), source_before);
    assert_eq!(session.source_revision(), revision_before);
}
