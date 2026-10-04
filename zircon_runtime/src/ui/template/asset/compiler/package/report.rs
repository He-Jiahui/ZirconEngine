use zircon_runtime_interface::ui::template::{
    UiActionPolicyReport, UiBindingPackageLifecycleStage, UiCompiledAssetDependencyManifest,
    UiCompiledAssetHeader, UiCompiledAssetPackageProfile, UiCompiledAssetPackageSection,
    UiCompiledAssetPackageValidationReport, UiInvalidationReport, UiLocalizationReport,
};

// 报告停在编译生命周期；它描述宿主需检查的策略、依赖及失效，不代表绑定已经被加载、执行或应用。
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
        binding_lifecycle_stage: UiBindingPackageLifecycleStage::Compiled,
        invalidation_report,
        action_policy_report,
        localization_report,
    }
}

// profile 此处只影响报告分区和上层动作策略，未按分区标签直接裁剪或扩充模板载荷。
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
