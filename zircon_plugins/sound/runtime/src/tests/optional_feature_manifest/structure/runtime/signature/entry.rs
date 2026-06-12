use super::super::super::sources::*;

#[test]
fn optional_feature_runtime_signature_entry_owns_full_assembly() {
    assert!(
        RUNTIME_SIGNATURE_ENTRY.contains("StaticOptionalFeatureManifest")
            && RUNTIME_SIGNATURE_ENTRY.contains("id: super::super::identity::feature_id(feature)")
            && RUNTIME_SIGNATURE_ENTRY
                .contains("display_name: super::super::identity::feature_display_name(feature)")
            && RUNTIME_SIGNATURE_ENTRY.contains(
                "owner_plugin_id: super::super::identity::feature_owner_plugin_id(feature)"
            )
            && RUNTIME_SIGNATURE_ENTRY.contains(
                "capabilities: super::super::capabilities::capability_signatures(feature)"
            )
            && RUNTIME_SIGNATURE_ENTRY
                .contains("default_packaging: super::super::defaults::default_packaging(feature)")
            && RUNTIME_SIGNATURE_ENTRY.contains(
                "enabled_by_default: super::super::defaults::enabled_by_default(feature)"
            )
            && RUNTIME_SIGNATURE_ENTRY.contains(
                "dependencies: super::super::dependencies::dependency_signatures(feature)"
            )
            && RUNTIME_SIGNATURE_ENTRY
                .contains("modules: super::super::modules::module_signatures(feature)"),
        "runtime signature entry child should own full optional-feature signature assembly"
    );
}
