use super::*;

#[test]
fn native_binding_owner_mismatch_is_rejected_at_store_namespace_boundary() {
    let plugin_id = PluginContributionId::parse("sample").unwrap();
    let command_id = EditorOperationPath::parse("plugin.sample.command").unwrap();

    let error = validate_native_binding_owner(&plugin_id, &command_id, "other")
        .expect_err("a callback from another plugin must not enter the contribution store");
    assert!(matches!(
        error,
        ContributionError::NativeBindingOwner {
            plugin_id: owner,
            command_id: id,
            binding_plugin_id,
        } if owner == plugin_id && id == command_id && binding_plugin_id == "other"
    ));
    assert!(validate_native_binding_owner(&plugin_id, &command_id, "sample").is_ok());

    let builtin_error =
        validate_native_binding_source(&ContributionSource::Builtin, &command_id, "sample")
            .expect_err("native callbacks must be attached to a plugin contribution source");
    assert!(matches!(
        builtin_error,
        ContributionError::NativeBindingRequiresPluginSource {
            command_id: id,
            binding_plugin_id,
        } if id == command_id && binding_plugin_id == "sample"
    ));
}
