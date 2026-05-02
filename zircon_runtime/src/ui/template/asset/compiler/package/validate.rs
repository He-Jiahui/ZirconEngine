use crate::ui::template::{
    collect_document_localization_report, validate_document_action_policy, UiCompileCacheKey,
    UiCompiledAssetArtifact, UiCompiledDocument, UiDocumentCompiler, UiInvalidationGraph,
};
use zircon_runtime_interface::ui::template::{
    UiActionHostPolicy, UiAssetDocument, UiAssetError, UiCompiledAssetPackageProfile,
    UiCompiledAssetPackageValidationReport,
};

use super::header::compiled_asset_header_from_cache_key;
use super::manifest::compiled_asset_dependency_manifest_from_imports;
use super::report::build_package_validation_report;

impl UiDocumentCompiler {
    pub fn validate_package(
        &self,
        document: &UiAssetDocument,
        profile: UiCompiledAssetPackageProfile,
    ) -> Result<UiCompiledAssetPackageValidationReport, UiAssetError> {
        let (_, report) = self.compile_package(document, profile)?;
        Ok(report)
    }

    pub fn compile_package_artifact(
        &self,
        document: &UiAssetDocument,
        profile: UiCompiledAssetPackageProfile,
    ) -> Result<UiCompiledAssetArtifact, UiAssetError> {
        let (compiled, report) = self.compile_package(document, profile)?;
        Ok(UiCompiledAssetArtifact::from_report_and_compiled(
            report, compiled,
        ))
    }

    fn compile_package(
        &self,
        document: &UiAssetDocument,
        profile: UiCompiledAssetPackageProfile,
    ) -> Result<(UiCompiledDocument, UiCompiledAssetPackageValidationReport), UiAssetError> {
        self.validate_compiler_preconditions(document)?;
        let cache_key = UiCompileCacheKey::from_compiler(self, document)?;
        let invalidation_snapshot = cache_key.invalidation_snapshot();
        let compiled = self.compile(document)?;
        let localization_report = collect_document_localization_report(document);

        let header = compiled_asset_header_from_cache_key(document.asset.clone(), cache_key);
        let dependencies = compiled_asset_dependency_manifest_from_imports(
            document,
            &header.compile_cache_key,
            &self.widget_imports,
            &self.style_imports,
            compiled.resource_dependencies(),
            &localization_report.dependencies,
        );
        let invalidation_report =
            UiInvalidationGraph::classify(None, &invalidation_snapshot, document);
        let action_policy = match profile {
            UiCompiledAssetPackageProfile::Runtime => UiActionHostPolicy::runtime_default(),
            UiCompiledAssetPackageProfile::Editor => UiActionHostPolicy::editor_authoring(),
        };
        let action_policy_report = validate_document_action_policy(document, &action_policy);

        let report = build_package_validation_report(
            profile,
            header,
            dependencies,
            invalidation_report,
            action_policy_report,
            localization_report,
        );
        Ok((compiled, report))
    }
}
