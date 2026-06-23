use super::*;

#[test]
fn timeline_editor_feature_descriptor_matches_runtime_feature_contract() {
    let feature = editor_feature();
    let descriptor = zircon_editor::EditorPlugin::descriptor(&feature);

    assert_eq!(descriptor.package_id, FEATURE_ID);
    assert_eq!(descriptor.display_name, "Sound Timeline Animation Track");
    assert_eq!(
        descriptor.crate_name,
        "zircon_plugin_sound_timeline_animation_editor"
    );
    assert_eq!(descriptor.capabilities, vec![CAPABILITY.to_string()]);
    assert_eq!(editor_capabilities(), vec![CAPABILITY.to_string()]);
}

#[test]
fn timeline_editor_feature_manifest_matches_runtime_provider_manifest() {
    assert_eq!(
        feature_manifest(),
        zircon_plugin_sound_timeline_animation_runtime::feature_manifest()
    );
}
