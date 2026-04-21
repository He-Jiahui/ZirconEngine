use super::support::*;

#[test]
fn generated_legacy_template_asset_source_opens_in_ui_asset_editor_session() {
    let document = UiTemplateLoader::load_toml_str(SIMPLE_LEGACY_TEMPLATE_TOML).unwrap();
    let source = UiLegacyTemplateAdapter::layout_source(
        "generated.legacy",
        "Generated Legacy Layout",
        &document,
    )
    .unwrap();
    let route = UiAssetEditorRoute::new(
        "generated.legacy.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );

    let session =
        UiAssetEditorSession::from_source(route, source, UiSize::new(1280.0, 720.0)).unwrap();

    assert!(session.diagnostics().is_empty());
    assert_eq!(
        session
            .reflection_model()
            .selection
            .primary_node_id
            .as_deref(),
        Some("root")
    );
}
