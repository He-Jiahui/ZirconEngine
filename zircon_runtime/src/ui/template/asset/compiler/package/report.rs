use zircon_runtime_interface::ui::template::{
    UiActionPolicyReport, UiCompiledAssetDependencyManifest, UiCompiledAssetHeader,
    UiCompiledAssetPackageProfile, UiCompiledAssetPackageSection,
    UiCompiledAssetPackageValidationReport, UiInvalidationReport, UiLocalizationReport,
};

pub(super) fn build_package_validation_report(
    profile: UiCompiledAssetPackageProfile,
    header: UiCompiledAssetHeader,
    dependencies: UiCompiledAssetDependencyManifest,
    invalidation_report: UiInvalidationReport,
    action_policy_report: UiActionPolicyReport,
    localization_report: UiLocalizationReport,
) -> UiCompiledAssetPackageValidationReport {
    let (retained_sections, stripped_sections) = profile_sections(profile);
    UiCompiledAssetPackageValidationReport {
        profile,
        header,
        dependencies,
        retained_sections,
        stripped_sections,
        invalidation_report,
        action_policy_report,
        localization_report,
    }
}

fn profile_sections(
    profile: UiCompiledAssetPackageProfile,
) -> (
    Vec<UiCompiledAssetPackageSection>,
    Vec<UiCompiledAssetPackageSection>,
) {
    match profile {
        UiCompiledAssetPackageProfile::Runtime => (
            vec![
                UiCompiledAssetPackageSection::RuntimeTemplateTree,
                UiCompiledAssetPackageSection::RuntimeStyleValues,
                UiCompiledAssetPackageSection::RuntimeBindings,
            ],
            vec![
                UiCompiledAssetPackageSection::SourceDocument,
                UiCompiledAssetPackageSection::AuthoringDiagnostics,
                UiCompiledAssetPackageSection::MigrationReport,
            ],
        ),
        UiCompiledAssetPackageProfile::Editor => (
            vec![
                UiCompiledAssetPackageSection::RuntimeTemplateTree,
                UiCompiledAssetPackageSection::RuntimeStyleValues,
                UiCompiledAssetPackageSection::RuntimeBindings,
                UiCompiledAssetPackageSection::SourceDocument,
                UiCompiledAssetPackageSection::AuthoringDiagnostics,
                UiCompiledAssetPackageSection::MigrationReport,
            ],
            Vec::new(),
        ),
    }
}
