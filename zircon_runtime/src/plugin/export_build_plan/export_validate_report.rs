use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use zircon_runtime_interface::export::ExportStage;

use super::{
    ExportBuildPlan, ExportGeneratedFile, LibraryEmbedCompileHostPlan,
    NativeDynamicPackageExportPlan, SourceTemplateBuildValidationPlan,
};
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{
    ExportBuildMode, ExportPackagingStrategy, ExportTargetPlatform,
};
use crate::plugin::RuntimePluginAvailabilityReport;

const EXPORT_VALIDATE_REPORT_SCHEMA_VERSION: u32 = 2;
const EXPORT_VALIDATE_CONTENTS_ARTIFACT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportValidateReport {
    pub stage: ExportStage,
    pub schema_version: u32,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_contents_artifact_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_contents_artifact_byte_length: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_contents_artifact_digest: Option<String>,
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
    pub byte_length: u64,
    pub content_digest: String,
}

impl ExportValidateReport {
    pub fn from_build_plan(
        project_manifest: impl Into<String>,
        stage_output: Option<String>,
        plan: &ExportBuildPlan,
    ) -> Self {
        let fatal_diagnostics = dedupe(plan.effective_fatal_diagnostics());
        Self {
            stage: ExportStage::Validate,
            schema_version: EXPORT_VALIDATE_REPORT_SCHEMA_VERSION,
            project_manifest: project_manifest.into(),
            profile: plan.profile.name.clone(),
            stage_output,
            profile_found: true,
            fatal: !fatal_diagnostics.is_empty(),
            diagnostics: dedupe(plan.diagnostics.clone()),
            fatal_diagnostics,
            profile_summary: Some(ExportValidateProfileSummary::from_build_plan(plan)),
            plan_summary: Some(ExportValidatePlanSummary::from_build_plan(plan)),
            generated_contents_artifact_path: None,
            generated_contents_artifact_byte_length: None,
            generated_contents_artifact_digest: None,
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
            stage: ExportStage::Validate,
            schema_version: EXPORT_VALIDATE_REPORT_SCHEMA_VERSION,
            project_manifest: project_manifest.into(),
            profile: profile.into(),
            stage_output,
            profile_found,
            fatal: true,
            diagnostics: vec![diagnostic.clone()],
            fatal_diagnostics: vec![diagnostic],
            profile_summary: None,
            plan_summary: None,
            generated_contents_artifact_path: None,
            generated_contents_artifact_byte_length: None,
            generated_contents_artifact_digest: None,
        }
    }

    pub fn generated_contents_artifact_json(
        plan: &ExportBuildPlan,
        pretty: bool,
    ) -> Result<String, serde_json::Error> {
        let artifact = ExportValidateContentsArtifact::from_build_plan(plan);
        if pretty {
            serde_json::to_string_pretty(&artifact)
        } else {
            serde_json::to_string(&artifact)
        }
    }

    pub fn record_generated_contents_artifact(
        &mut self,
        path: String,
        byte_length: u64,
        digest: String,
    ) {
        self.generated_contents_artifact_path = Some(path);
        self.generated_contents_artifact_byte_length = Some(byte_length);
        self.generated_contents_artifact_digest = Some(digest);
    }

    pub fn sha256_digest(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
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
            byte_length: file.contents.len() as u64,
            content_digest: ExportValidateReport::sha256_digest(file.contents.as_bytes()),
        }
    }
}

#[derive(Serialize)]
struct ExportValidateContentsArtifact<'a> {
    schema_version: u32,
    generated_files: Vec<ExportValidateContentsArtifactFile<'a>>,
}

impl<'a> ExportValidateContentsArtifact<'a> {
    fn from_build_plan(plan: &'a ExportBuildPlan) -> Self {
        Self {
            schema_version: EXPORT_VALIDATE_CONTENTS_ARTIFACT_SCHEMA_VERSION,
            generated_files: plan
                .generated_files
                .iter()
                .map(ExportValidateContentsArtifactFile::from_generated_file)
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct ExportValidateContentsArtifactFile<'a> {
    path: &'a str,
    purpose: &'a str,
    contents: &'a str,
}

impl<'a> ExportValidateContentsArtifactFile<'a> {
    fn from_generated_file(file: &'a ExportGeneratedFile) -> Self {
        Self {
            path: &file.path,
            purpose: &file.purpose,
            contents: &file.contents,
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
