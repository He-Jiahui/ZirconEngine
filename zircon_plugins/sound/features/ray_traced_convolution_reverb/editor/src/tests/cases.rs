use super::*;

#[test]
fn ray_traced_editor_feature_descriptor_matches_runtime_feature_contract() {
    let feature = editor_feature();
    let descriptor = zircon_editor::EditorPlugin::descriptor(&feature);

    assert_eq!(descriptor.package_id, FEATURE_ID);
    assert_eq!(
        descriptor.display_name,
        "Sound Ray-Traced Convolution Reverb"
    );
    assert_eq!(
        descriptor.crate_name,
        "zircon_plugin_sound_ray_traced_convolution_editor"
    );
    assert_eq!(descriptor.capabilities, vec![CAPABILITY.to_string()]);
    assert_eq!(editor_capabilities(), vec![CAPABILITY.to_string()]);
}

#[test]
fn ray_traced_editor_feature_manifest_matches_runtime_provider_manifest() {
    assert_eq!(
        feature_manifest(),
        zircon_plugin_sound_ray_traced_convolution_runtime::feature_manifest()
    );
}
