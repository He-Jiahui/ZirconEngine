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
    #[serde(default)]
    pub binding_lifecycle_stage: UiBindingPackageLifecycleStage,
    pub invalidation_report: UiInvalidationReport,
    pub action_policy_report: UiActionPolicyReport,
    pub localization_report: UiLocalizationReport,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingPackageLifecycleStage {
    #[default]
    Declared,
    Compiled,
    Loaded,
    Bound,
    Executed,
    Applied,
}

impl UiBindingPackageLifecycleStage {
    pub const ALL: [Self; 6] = [
        Self::Declared,
        Self::Compiled,
        Self::Loaded,
        Self::Bound,
        Self::Executed,
        Self::Applied,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::Compiled => "compiled",
            Self::Loaded => "loaded",
            Self::Bound => "bound",
            Self::Executed => "executed",
            Self::Applied => "applied",
        }
    }
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
