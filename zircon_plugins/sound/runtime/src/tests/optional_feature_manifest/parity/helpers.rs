use super::super::support::{optional_feature_signature, StaticOptionalFeatureManifest};

pub(super) fn sorted_static_optional_feature_signatures(
    mut signatures: Vec<StaticOptionalFeatureManifest>,
) -> Vec<StaticOptionalFeatureManifest> {
    signatures.sort_unstable_by(|left, right| left.id().cmp(right.id()));
    signatures
}

pub(super) fn sorted_runtime_optional_feature_signatures(
    features: impl IntoIterator<Item = zircon_runtime::plugin::PluginFeatureBundleManifest>,
) -> Vec<StaticOptionalFeatureManifest> {
    let mut signatures = features
        .into_iter()
        .map(|feature| optional_feature_signature(&feature))
        .collect::<Vec<_>>();
    signatures.sort_unstable_by(|left, right| left.id().cmp(right.id()));
    signatures
}

pub(super) fn sorted_registration_report_optional_feature_signatures(
    reports: impl IntoIterator<Item = zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport>,
) -> Vec<StaticOptionalFeatureManifest> {
    let mut signatures = reports
        .into_iter()
        .map(|report| {
            assert!(report.is_success(), "{:?}", report.diagnostics);
            assert_eq!(report.provider_package_id_or_owner(), "sound");
            optional_feature_signature(&report.manifest)
        })
        .collect::<Vec<_>>();
    signatures.sort_unstable_by(|left, right| left.id().cmp(right.id()));
    signatures
}

pub(super) fn assert_feature_registration_module(
    report: zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport,
    expected_name: &str,
    expected_description: &str,
) {
    assert!(report.is_success(), "{:?}", report.diagnostics);
    let modules = report.extensions.modules();
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].name, expected_name);
    assert_eq!(modules[0].description, expected_description);
}
