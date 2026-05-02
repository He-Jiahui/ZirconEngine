use serde::{Deserialize, Serialize};

use crate::ui::template::{UiActionPolicyReport, UiInvalidationReport, UiLocalizationReport};

use super::{
    UiCompiledAssetDependencyManifest, UiCompiledAssetHeader, UiCompiledAssetPackageProfile,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiCompiledAssetPackageValidationReport {
    pub profile: UiCompiledAssetPackageProfile,
    pub header: UiCompiledAssetHeader,
    pub dependencies: UiCompiledAssetDependencyManifest,
    pub retained_sections: Vec<UiCompiledAssetPackageSection>,
    pub stripped_sections: Vec<UiCompiledAssetPackageSection>,
    pub invalidation_report: UiInvalidationReport,
    pub action_policy_report: UiActionPolicyReport,
    pub localization_report: UiLocalizationReport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiCompiledAssetPackageSection {
    RuntimeTemplateTree,
    RuntimeStyleValues,
    RuntimeBindings,
    SourceDocument,
    AuthoringDiagnostics,
    MigrationReport,
}
