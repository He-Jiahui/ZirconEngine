use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{
    ExportBuildPlan, ExportGeneratedFile, LibraryEmbedCompileHostPlan,
    NativeDynamicPackageExportPlan, SourceTemplateBuildValidationPlan,
};
use crate::builtin::RuntimeTargetMode;
use crate::plugin::{
    ExportBuildMode, ExportPackagingStrategy, ExportTargetPlatform, RuntimePluginAvailabilityReport,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ExportPipelineStage {
    Validate,
    SourceTemplate,
    NativeDynamic,
    CompileHost,
    CookAssets,
    Pack,
    PlatformBundle,
    Report,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportValidateReport {
    pub stage: ExportPipelineStage,
    pub project_manifest: String,
    pub profile: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_output: Option<String>,
    pub profile_found: bool,
    pub fatal: bool,
    pub diagnostics: Vec<String>,
    pub fatal_diagnostics: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_summary: Option<ExportValidateProfileSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_summary: Option<ExportValidatePlanSummary>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportValidateProfileSummary {
    pub name: String,
    pub target_mode: RuntimeTargetMode,
    pub target_platform: ExportTargetPlatform,
    pub build_mode: ExportBuildMode,
    pub strategies: Vec<ExportPackagingStrategy>,
    pub selected_plugins: Vec<String>,
    pub features: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_filter: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportValidatePlanSummary {
    pub enabled_runtime_plugins: Vec<String>,
    pub linked_runtime_crates: Vec<String>,
    pub native_dynamic_packages: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub native_dynamic_package_exports: Vec<NativeDynamicPackageExportPlan>,
    pub generated_files: Vec<ExportValidateGeneratedFileSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_embed_compile_host: Option<LibraryEmbedCompileHostPlan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_template_build: Option<SourceTemplateBuildValidationPlan>,
    pub runtime_plugin_availability: RuntimePluginAvailabilityReport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportValidateGeneratedFileSummary {
    pub path: String,
    pub purpose: String,
    pub contents: String,
}

impl ExportValidateReport {
    pub fn from_build_plan(
        project_manifest: impl Into<String>,
        stage_output: Option<String>,
        plan: &ExportBuildPlan,
    ) -> Self {
        let fatal_diagnostics = dedupe(plan.effective_fatal_diagnostics());
        Self {
            stage: ExportPipelineStage::Validate,
            project_manifest: project_manifest.into(),
            profile: plan.profile.name.clone(),
            stage_output,
            profile_found: true,
            fatal: !fatal_diagnostics.is_empty(),
            diagnostics: dedupe(plan.diagnostics.clone()),
            fatal_diagnostics,
            profile_summary: Some(ExportValidateProfileSummary::from_build_plan(plan)),
            plan_summary: Some(ExportValidatePlanSummary::from_build_plan(plan)),
        }
    }

    pub fn fatal_error(
        project_manifest: impl Into<String>,
        profile: impl Into<String>,
        stage_output: Option<String>,
        profile_found: bool,
        diagnostic: impl Into<String>,
    ) -> Self {
        let diagnostic = diagnostic.into();
        Self {
            stage: ExportPipelineStage::Validate,
            project_manifest: project_manifest.into(),
            profile: profile.into(),
            stage_output,
            profile_found,
            fatal: true,
            diagnostics: vec![diagnostic.clone()],
            fatal_diagnostics: vec![diagnostic],
            profile_summary: None,
            plan_summary: None,
        }
    }
}

impl ExportValidateProfileSummary {
    fn from_build_plan(plan: &ExportBuildPlan) -> Self {
        Self {
            name: plan.profile.name.clone(),
            target_mode: plan.profile.target_mode,
            target_platform: plan.profile.target_platform,
            build_mode: plan.profile.build_mode,
            strategies: plan.profile.strategies.clone(),
            selected_plugins: plan.profile.selected_plugins.clone(),
            features: plan.profile.features.clone(),
            asset_filter: plan.profile.asset_filter.clone(),
        }
    }
}

impl ExportValidatePlanSummary {
    fn from_build_plan(plan: &ExportBuildPlan) -> Self {
        Self {
            enabled_runtime_plugins: plan.enabled_runtime_plugins.clone(),
            linked_runtime_crates: plan.linked_runtime_crates.clone(),
            native_dynamic_packages: plan.native_dynamic_packages.clone(),
            native_dynamic_package_exports: plan.native_dynamic_package_exports.clone(),
            generated_files: plan
                .generated_files
                .iter()
                .map(ExportValidateGeneratedFileSummary::from_generated_file)
                .collect(),
            library_embed_compile_host: plan.library_embed_compile_host.clone(),
            source_template_build: plan.source_template_build.clone(),
            runtime_plugin_availability: plan.runtime_plugin_availability.clone(),
        }
    }
}

impl ExportValidateGeneratedFileSummary {
    fn from_generated_file(file: &ExportGeneratedFile) -> Self {
        Self {
            path: file.path.clone(),
            purpose: file.purpose.clone(),
            contents: file.contents.clone(),
        }
    }
}

fn dedupe(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.iter().any(|existing| existing == &value) {
            deduped.push(value);
        }
    }
    deduped
}
