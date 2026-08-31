use serde::{Deserialize, Serialize};

use super::{
    management::{RenderMaterialManagementRecord, RenderMaterialManagementSnapshot},
    RenderMaterialDependencySet, RenderMaterialDiagnosticSource, RenderMaterialFallbackPolicy,
    RenderMaterialPropertyUniformField, RenderMaterialPropertyUniformSummary,
    RenderMaterialPropertyUniformUnsupported, RenderMaterialPropertyValue,
    RenderMaterialPropertyValueState, RenderMaterialPropertyValueSummary,
    RenderMaterialTextureSlotState, RenderMaterialTextureSlotSummary,
    RenderMaterialValidationError,
};
use crate::core::resource::{AssetReference, ResourceId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialFallbackUsage {
    pub reason: RenderMaterialFallbackReason,
    pub fallback_policy: RenderMaterialFallbackPolicy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum RenderMaterialFallbackReason {
    Material {
        material: ResourceId,
    },
    Shader {
        reference: AssetReference,
    },
    Texture {
        slot: String,
        reference: AssetReference,
    },
    Validation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialReadinessDiagnostic {
    pub source: RenderMaterialDiagnosticSource,
    pub path: String,
    pub diagnostic: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialReadinessReport {
    pub material_name: Option<String>,
    pub dependencies: RenderMaterialDependencySet,
    pub fallback_policy: RenderMaterialFallbackPolicy,
    pub validation_errors: Vec<RenderMaterialValidationError>,
    pub fallback_usages: Vec<RenderMaterialFallbackUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property_value_summary: Option<RenderMaterialPropertyValueSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub property_value_states: Vec<RenderMaterialPropertyValueState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uniform_summary: Option<RenderMaterialPropertyUniformSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uniform_fields: Vec<RenderMaterialPropertyUniformField>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uniform_unsupported: Vec<RenderMaterialPropertyUniformUnsupported>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard_texture_slot_summary: Option<RenderMaterialTextureSlotSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub standard_texture_slot_states: Vec<RenderMaterialTextureSlotState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_slot_summary: Option<RenderMaterialTextureSlotSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub non_standard_texture_slot_states: Vec<RenderMaterialTextureSlotState>,
    #[serde(default)]
    pub diagnostics: Vec<RenderMaterialReadinessDiagnostic>,
}

/// Compact UI/API classification derived from readiness issue severity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMaterialReadinessStatus {
    #[default]
    Ready,
    Diagnostic,
    Fallback,
    Invalid,
}

impl RenderMaterialReadinessStatus {
    pub fn from_issue_counts(
        validation_error_count: usize,
        fallback_usage_count: usize,
        diagnostic_count: usize,
    ) -> Self {
        if validation_error_count > 0 {
            Self::Invalid
        } else if fallback_usage_count > 0 {
            Self::Fallback
        } else if diagnostic_count > 0 {
            Self::Diagnostic
        } else {
            Self::Ready
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialReadinessSummary {
    #[serde(default)]
    pub status: RenderMaterialReadinessStatus,
    pub is_ready: bool,
    pub uses_fallback: bool,
    pub has_diagnostics: bool,
    pub validation_error_count: usize,
    pub fallback_usage_count: usize,
    pub diagnostic_count: usize,
    pub property_value_summary: Option<RenderMaterialPropertyValueSummary>,
    pub uniform_summary: Option<RenderMaterialPropertyUniformSummary>,
    pub standard_texture_slot_summary: Option<RenderMaterialTextureSlotSummary>,
    pub texture_slot_summary: Option<RenderMaterialTextureSlotSummary>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialIssueState {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation_errors: Vec<RenderMaterialValidationError>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallback_usages: Vec<RenderMaterialFallbackUsage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<RenderMaterialReadinessDiagnostic>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialPreparedState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property_value_summary: Option<RenderMaterialPropertyValueSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub property_value_states: Vec<RenderMaterialPropertyValueState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uniform_summary: Option<RenderMaterialPropertyUniformSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uniform_fields: Vec<RenderMaterialPropertyUniformField>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uniform_unsupported: Vec<RenderMaterialPropertyUniformUnsupported>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard_texture_slot_summary: Option<RenderMaterialTextureSlotSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub standard_texture_slot_states: Vec<RenderMaterialTextureSlotState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_slot_summary: Option<RenderMaterialTextureSlotSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub non_standard_texture_slot_states: Vec<RenderMaterialTextureSlotState>,
}

impl RenderMaterialIssueState {
    pub fn status(&self) -> RenderMaterialReadinessStatus {
        RenderMaterialReadinessStatus::from_issue_counts(
            self.validation_errors.len(),
            self.fallback_usages.len(),
            self.diagnostics.len(),
        )
    }

    pub fn is_ready(&self) -> bool {
        self.validation_errors.is_empty() && self.fallback_usages.is_empty()
    }

    pub fn uses_fallback(&self) -> bool {
        !self.fallback_usages.is_empty()
    }

    pub fn has_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}

impl RenderMaterialReadinessReport {
    pub fn is_ready(&self) -> bool {
        self.validation_errors.is_empty() && self.fallback_usages.is_empty()
    }

    pub fn uses_fallback(&self) -> bool {
        !self.fallback_usages.is_empty()
    }

    pub fn has_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    pub fn status(&self) -> RenderMaterialReadinessStatus {
        RenderMaterialReadinessStatus::from_issue_counts(
            self.validation_errors.len(),
            self.fallback_usages.len(),
            self.diagnostics.len(),
        )
    }

    pub fn summary(&self) -> RenderMaterialReadinessSummary {
        RenderMaterialReadinessSummary {
            status: self.status(),
            is_ready: self.is_ready(),
            uses_fallback: self.uses_fallback(),
            has_diagnostics: self.has_diagnostics(),
            validation_error_count: self.validation_errors.len(),
            fallback_usage_count: self.fallback_usages.len(),
            diagnostic_count: self.diagnostics.len(),
            property_value_summary: self.property_value_summary,
            uniform_summary: self.uniform_summary,
            standard_texture_slot_summary: self.standard_texture_slot_summary,
            texture_slot_summary: self.texture_slot_summary,
        }
    }

    pub fn issue_state(&self) -> RenderMaterialIssueState {
        RenderMaterialIssueState {
            validation_errors: self.validation_errors.clone(),
            fallback_usages: self.fallback_usages.clone(),
            diagnostics: self.diagnostics.clone(),
        }
    }

    pub fn prepared_state(&self) -> RenderMaterialPreparedState {
        RenderMaterialPreparedState {
            property_value_summary: self.property_value_summary,
            property_value_states: self.property_value_states.clone(),
            uniform_summary: self.uniform_summary,
            uniform_fields: self.uniform_fields.clone(),
            uniform_unsupported: self.uniform_unsupported.clone(),
            standard_texture_slot_summary: self.standard_texture_slot_summary,
            standard_texture_slot_states: self.standard_texture_slot_states.clone(),
            texture_slot_summary: self.texture_slot_summary,
            non_standard_texture_slot_states: self.non_standard_texture_slot_states.clone(),
        }
    }

    pub fn management_snapshot(&self) -> RenderMaterialManagementSnapshot {
        RenderMaterialManagementSnapshot {
            summary: self.summary(),
            issue_state: self.issue_state(),
            prepared_state: self.prepared_state(),
        }
    }

    pub fn management_record(&self, material_id: ResourceId) -> RenderMaterialManagementRecord {
        RenderMaterialManagementRecord {
            material_id,
            material_name: self.material_name.clone(),
            snapshot: self.management_snapshot(),
        }
    }

    pub fn push_validation_error_once(&mut self, error: RenderMaterialValidationError) {
        push_unique_with_recent_fast_path(&mut self.validation_errors, error);
    }

    pub fn push_fallback_usage_once(&mut self, usage: RenderMaterialFallbackUsage) {
        push_unique_with_recent_fast_path(&mut self.fallback_usages, usage);
    }

    pub fn push_diagnostic_once(&mut self, diagnostic: RenderMaterialReadinessDiagnostic) {
        push_unique_with_recent_fast_path(&mut self.diagnostics, diagnostic);
    }
}

fn push_unique_with_recent_fast_path<T: PartialEq>(items: &mut Vec<T>, item: T) {
    if items.last() == Some(&item) || items.contains(&item) {
        return;
    }
    items.push(item);
}

#[cfg(test)]
#[path = "readiness_report/recent_duplicate_fast_path_tests.rs"]
mod recent_duplicate_fast_path_tests;

#[cfg(test)]
mod tests;
