pub(super) const STATIC_PLUGIN_MANIFESTS: &[(&str, &str)] = &[
    // @cargo-zircon:static-manifest-begin
    ("ai", include_str!("../../../ai/plugin.toml")),
    (
        "animation_graph",
        include_str!("../../../animation_graph/plugin.toml"),
    ),
    ("animation", include_str!("../../../animation/plugin.toml")),
    (
        "audio_importer",
        include_str!("../../../audio_importer/plugin.toml"),
    ),
    (
        "asset_importer.audio",
        include_str!("../../../asset_importers/audio/plugin.toml"),
    ),
    (
        "asset_importer.data",
        include_str!("../../../asset_importers/data/plugin.toml"),
    ),
    (
        "asset_importer.model",
        include_str!("../../../asset_importers/model/plugin.toml"),
    ),
    (
        "asset_importer.shader",
        include_str!("../../../asset_importers/shader/plugin.toml"),
    ),
    (
        "asset_importer.texture",
        include_str!("../../../asset_importers/texture/plugin.toml"),
    ),
    (
        "editor_build_export_desktop",
        include_str!("../../../editor_build_export_desktop/plugin.toml"),
    ),
    (
        "gltf_importer",
        include_str!("../../../gltf_importer/plugin.toml"),
    ),
    ("hybrid_gi", include_str!("../../../hybrid_gi/plugin.toml")),
    (
        "material_editor",
        include_str!("../../../material_editor/plugin.toml"),
    ),
    (
        "native_window_hosting",
        include_str!("../../../native_window_hosting/plugin.toml"),
    ),
    (
        "navigation",
        include_str!("../../../navigation/plugin.toml"),
    ),
    ("net", include_str!("../../../net/plugin.toml")),
    (
        "obj_importer",
        include_str!("../../../obj_importer/plugin.toml"),
    ),
    (
        "opus_importer",
        include_str!("../../../opus_importer/plugin.toml"),
    ),
    ("particles", include_str!("../../../particles/plugin.toml")),
    ("physics", include_str!("../../../physics/plugin.toml")),
    (
        "plugin_sdk_examples",
        include_str!("../../../plugin_sdk_examples/plugin.toml"),
    ),
    (
        "prefab_tools",
        include_str!("../../../prefab_tools/plugin.toml"),
    ),
    ("rendering", include_str!("../../../rendering/plugin.toml")),
    (
        "runtime_diagnostics",
        include_str!("../../../runtime_diagnostics/plugin.toml"),
    ),
    (
        "shader_wgsl_importer",
        include_str!("../../../shader_wgsl_importer/plugin.toml"),
    ),
    ("solari", include_str!("../../../solari/plugin.toml")),
    ("sound", include_str!("../../../sound/plugin.toml")),
    ("terrain", include_str!("../../../terrain/plugin.toml")),
    (
        "texture_importer",
        include_str!("../../../texture_importer/plugin.toml"),
    ),
    ("texture", include_str!("../../../texture/plugin.toml")),
    (
        "tilemap_2d",
        include_str!("../../../tilemap_2d/plugin.toml"),
    ),
    (
        "timeline_sequence",
        include_str!("../../../timeline_sequence/plugin.toml"),
    ),
    (
        "ui_asset_authoring",
        include_str!("../../../ui_asset_authoring/plugin.toml"),
    ),
    (
        "ui_document_importer",
        include_str!("../../../ui_document_importer/plugin.toml"),
    ),
    (
        "virtual_geometry",
        include_str!("../../../virtual_geometry/plugin.toml"),
    ),
    (
        "zr_vm_language",
        include_str!("../../../zr_vm_language/plugin.toml"),
    ),
    // @cargo-zircon:static-manifest-end
];

#[test]
fn plugins_12_static_plugin_manifest_is_generated() {
    let mut missing_generated_headers = Vec::new();
    for (package_id, manifest_toml) in STATIC_PLUGIN_MANIFESTS {
        if !manifest_toml.starts_with(super::GENERATED_MANIFEST_HEADER) {
            missing_generated_headers.push(*package_id);
        }
        let decoded = super::parse_manifest(package_id, manifest_toml);
        assert_eq!(
            decoded.id, *package_id,
            "{package_id} plugin.toml id drifted"
        );
    }
    assert!(
        missing_generated_headers.is_empty(),
        "static plugin.toml files missing @generated header: {missing_generated_headers:?}"
    );

    #[cfg(any(
        feature = "base-runtime-plugins",
        feature = "advanced-render-runtime-plugins",
        feature = "navigation-runtime-plugin",
        feature = "zr-vm-language-runtime-plugin"
    ))]
    super::assert_runtime_descriptor_manifests_match_generated_static_manifests();
    super::assert_native_dynamic_fixture_manifest_is_sdk_declared();
}
