#[test]
fn review_f8_texture_import_settings_use_fallible_apply_not_with() {
    let descriptor = include_str!("../../../../asset/assets/texture/descriptor.rs");
    let descriptor_settings =
        include_str!("../../../../asset/assets/texture/descriptor/settings.rs");
    let texture_asset = include_str!("../../../../asset/assets/texture/texture_asset.rs");
    let runtime_importer = include_str!("../../../../asset/importer/ingest/import_texture.rs");
    let plugin_importer =
        include_str!("../../../../../../zircon_plugins/texture_importer/runtime/src/importers.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_04_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let importer_doc = include_str!("../../../../../../docs/zircon_runtime/asset/importer.md");
    let render_asset_doc =
        include_str!("../../../../../../docs/zircon_runtime/asset/render-assets.md");

    let old_fallible_with_name = ["with", "import", "settings"].join("_");
    for (name, source) in [
        ("TextureAssetDescriptor", descriptor),
        ("TextureAsset", texture_asset),
        ("runtime texture importer", runtime_importer),
        ("texture importer plugin", plugin_importer),
    ] {
        assert!(
            source.contains("apply_import_settings"),
            "F8 texture import settings source `{name}` should use the fallible apply_* API"
        );
        assert!(
            !source.contains(&old_fallible_with_name),
            "F8 texture import settings source `{name}` should not keep the fallible with_* API"
        );
    }
    assert!(
        descriptor.contains("pub type TextureDescriptorResult<T> = std::result::Result<T,")
            && descriptor.contains("pub enum TextureDescriptorError")
            && descriptor.contains(") -> TextureDescriptorResult<Self>")
            && texture_asset.contains(") -> TextureDescriptorResult<Self>"),
        "Texture import settings should remain fallible, typed, and no longer use a builder-style with_* verb"
    );
    for forbidden in [
        "pub fn apply_import_settings(mut self, settings: &toml::Table) -> Result<Self, String>",
        ") -> Result<Self, String>",
        ") -> Result<(), String>",
        "Err(format!(",
    ] {
        assert!(
            !descriptor.contains(forbidden)
                && !descriptor_settings.contains(forbidden)
                && !texture_asset.contains(forbidden),
            "F8 texture descriptor apply API should not keep `{forbidden}`"
        );
    }
    assert!(
        texture_asset.contains(".apply_import_settings(settings)?")
            && runtime_importer.contains(".apply_import_settings(&context.import_settings)")
            && plugin_importer.contains(".apply_import_settings(&context.import_settings)"),
        "Runtime and plugin importers should call the fallible apply_import_settings entry"
    );

    for doc_anchor in [
        "F8 texture import settings apply API",
        "texture_import_settings_apply_api_coremin_check_passed",
        "review_f8_texture_import_settings_use_fallible_apply_not_with",
        "apply_import_settings",
        "TextureDescriptorError",
        "runtime_15_texture_descriptor_typed_errors_static_passed_cargo_deferred",
        "RuntimePluginDescriptor public-field convergence complete",
        "RuntimePluginDescriptor::new retired",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_04_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || importer_doc.contains(doc_anchor)
                || render_asset_doc.contains(doc_anchor),
            "F8 texture import settings docs should record `{doc_anchor}`"
        );
    }
}
