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
        if !self.validation_errors.contains(&error) {
            self.validation_errors.push(error);
        }
    }

    pub fn push_fallback_usage_once(&mut self, usage: RenderMaterialFallbackUsage) {
        if !self.fallback_usages.contains(&usage) {
            self.fallback_usages.push(usage);
        }
    }

    pub fn push_diagnostic_once(&mut self, diagnostic: RenderMaterialReadinessDiagnostic) {
        if !self.diagnostics.contains(&diagnostic) {
            self.diagnostics.push(diagnostic);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::management::{
        RenderMaterialManagementOverview, RenderMaterialManagementPageRequest,
        RenderMaterialManagementQuery, RenderMaterialManagementRecord,
        RenderMaterialManagementRecordSet, RenderMaterialManagementRecordSummary,
        RenderMaterialManagementSortDirection, RenderMaterialManagementSortKey,
        RenderMaterialManagementSortOrder, RenderMaterialManagementStatusIndex,
        RenderMaterialManagementStatusView,
    };
    use super::*;
    use crate::core::framework::render::RenderMaterialPropertyUniformUnsupportedReason;
    use crate::core::resource::ResourceLocator;

    #[test]
    fn material_readiness_report_deduplicates_material_uniform_diagnostics() {
        let shader = AssetReference::from_locator(
            ResourceLocator::parse("res://shaders/uniform_material.zshader")
                .expect("valid shader locator"),
        );
        let mut report = RenderMaterialReadinessReport {
            material_name: Some("UniformMaterial".to_string()),
            dependencies: RenderMaterialDependencySet::new(shader),
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
            validation_errors: Vec::new(),
            fallback_usages: Vec::new(),
            property_value_summary: None,
            property_value_states: Vec::new(),
            uniform_summary: None,
            uniform_fields: Vec::new(),
            uniform_unsupported: Vec::new(),
            standard_texture_slot_summary: None,
            standard_texture_slot_states: Vec::new(),
            texture_slot_summary: None,
            non_standard_texture_slot_states: Vec::new(),
            diagnostics: Vec::new(),
        };
        let diagnostic = RenderMaterialReadinessDiagnostic {
            source: RenderMaterialDiagnosticSource::MaterialUniform,
            path: "uniform.debug_label".to_string(),
            diagnostic: "material property debug_label cannot be encoded into the renderer uniform payload: unsupported property type".to_string(),
        };

        report.push_diagnostic_once(diagnostic.clone());
        report.push_diagnostic_once(diagnostic);

        assert!(report.is_ready());
        assert!(report.has_diagnostics());
        assert_eq!(report.status(), RenderMaterialReadinessStatus::Diagnostic);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(
            report.diagnostics[0].source,
            RenderMaterialDiagnosticSource::MaterialUniform
        );
        assert_eq!(report.diagnostics[0].path, "uniform.debug_label");

        let issue_state = report.issue_state();
        assert_eq!(
            issue_state.status(),
            RenderMaterialReadinessStatus::Diagnostic
        );
        assert!(issue_state.is_ready());
        assert!(!issue_state.uses_fallback());
        assert!(issue_state.has_diagnostics());
        assert!(issue_state.validation_errors.is_empty());
        assert!(issue_state.fallback_usages.is_empty());
        assert_eq!(issue_state.diagnostics, report.diagnostics);

        let management_snapshot = report.management_snapshot();
        assert_eq!(
            management_snapshot.summary.status,
            RenderMaterialReadinessStatus::Diagnostic
        );
        assert_eq!(management_snapshot.summary, report.summary());
        assert_eq!(management_snapshot.issue_state, issue_state);
        assert_eq!(management_snapshot.prepared_state, report.prepared_state());
    }

    #[test]
    fn material_readiness_status_classifies_issue_severity() {
        let shader = AssetReference::from_locator(
            ResourceLocator::parse("res://shaders/status_material.zshader")
                .expect("valid shader locator"),
        );
        let mut report = RenderMaterialReadinessReport {
            material_name: Some("StatusMaterial".to_string()),
            dependencies: RenderMaterialDependencySet::new(shader.clone()),
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
            validation_errors: Vec::new(),
            fallback_usages: Vec::new(),
            property_value_summary: None,
            property_value_states: Vec::new(),
            uniform_summary: None,
            uniform_fields: Vec::new(),
            uniform_unsupported: Vec::new(),
            standard_texture_slot_summary: None,
            standard_texture_slot_states: Vec::new(),
            texture_slot_summary: None,
            non_standard_texture_slot_states: Vec::new(),
            diagnostics: Vec::new(),
        };

        assert_eq!(report.status(), RenderMaterialReadinessStatus::Ready);
        assert_eq!(
            report.issue_state().status(),
            RenderMaterialReadinessStatus::Ready
        );
        assert_eq!(
            report.summary().status,
            RenderMaterialReadinessStatus::Ready
        );

        report.push_diagnostic_once(RenderMaterialReadinessDiagnostic {
            source: RenderMaterialDiagnosticSource::MaterialAsset,
            path: "material.validation_diagnostics[0]".to_string(),
            diagnostic: "authoring note".to_string(),
        });
        assert_eq!(report.status(), RenderMaterialReadinessStatus::Diagnostic);
        assert!(report.is_ready());

        report.diagnostics.clear();
        report.push_fallback_usage_once(RenderMaterialFallbackUsage {
            reason: RenderMaterialFallbackReason::Shader { reference: shader },
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
        });
        assert_eq!(report.status(), RenderMaterialReadinessStatus::Fallback);
        assert!(!report.is_ready());

        report.push_validation_error_once(RenderMaterialValidationError::InvalidMaskCutoff {
            cutoff: 1.25,
        });
        assert_eq!(report.status(), RenderMaterialReadinessStatus::Invalid);
        assert_eq!(
            report.issue_state().status(),
            RenderMaterialReadinessStatus::Invalid
        );
        assert_eq!(
            report.summary().status,
            RenderMaterialReadinessStatus::Invalid
        );
    }

    #[test]
    fn material_readiness_report_summary_counts_status_and_prepared_summaries() {
        let shader = AssetReference::from_locator(
            ResourceLocator::parse("res://shaders/summary_material.zshader")
                .expect("valid shader locator"),
        );
        let mut report = RenderMaterialReadinessReport {
            material_name: Some("SummaryMaterial".to_string()),
            dependencies: RenderMaterialDependencySet::new(shader),
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
            validation_errors: vec![RenderMaterialValidationError::InvalidMaskCutoff {
                cutoff: 1.25,
            }],
            fallback_usages: vec![RenderMaterialFallbackUsage {
                reason: RenderMaterialFallbackReason::Validation,
                fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
            }],
            property_value_summary: Some(RenderMaterialPropertyValueSummary {
                total_count: 2,
                float_count: 1,
                string_count: 1,
                ..RenderMaterialPropertyValueSummary::default()
            }),
            property_value_states: vec![RenderMaterialPropertyValueState {
                name: "custom_gain".to_string(),
                value: RenderMaterialPropertyValue::Float { value: 2.5 },
            }],
            uniform_summary: Some(RenderMaterialPropertyUniformSummary {
                payload_byte_len: 16,
                field_count: 1,
                unsupported_count: 1,
            }),
            uniform_fields: vec![RenderMaterialPropertyUniformField {
                name: "custom_gain".to_string(),
                kind: "float".to_string(),
                offset: 0,
                size: 4,
                alignment: 4,
            }],
            uniform_unsupported: vec![RenderMaterialPropertyUniformUnsupported {
                name: "debug_label".to_string(),
                reason: RenderMaterialPropertyUniformUnsupportedReason::UnsupportedType,
            }],
            standard_texture_slot_summary: Some(RenderMaterialTextureSlotSummary {
                total_count: 2,
                resolved_count: 1,
                fallback_count: 1,
            }),
            standard_texture_slot_states: vec![
                RenderMaterialTextureSlotState {
                    slot: "base_color".to_string(),
                    texture_id: Some(ResourceId::from_stable_label("texture:base")),
                    fallback: None,
                },
                RenderMaterialTextureSlotState {
                    slot: "normal".to_string(),
                    texture_id: None,
                    fallback: None,
                },
            ],
            texture_slot_summary: Some(RenderMaterialTextureSlotSummary {
                total_count: 1,
                resolved_count: 0,
                fallback_count: 1,
            }),
            non_standard_texture_slot_states: vec![RenderMaterialTextureSlotState {
                slot: "mask_map".to_string(),
                texture_id: None,
                fallback: None,
            }],
            diagnostics: Vec::new(),
        };
        report.push_diagnostic_once(RenderMaterialReadinessDiagnostic {
            source: RenderMaterialDiagnosticSource::MaterialUniform,
            path: "uniform.debug_label".to_string(),
            diagnostic: "debug label is retained as metadata".to_string(),
        });

        let summary = report.summary();

        assert_eq!(report.status(), RenderMaterialReadinessStatus::Invalid);
        assert_eq!(summary.status, RenderMaterialReadinessStatus::Invalid);
        assert!(!summary.is_ready);
        assert!(summary.uses_fallback);
        assert!(summary.has_diagnostics);
        assert_eq!(summary.validation_error_count, 1);
        assert_eq!(summary.fallback_usage_count, 1);
        assert_eq!(summary.diagnostic_count, 1);
        assert_eq!(
            summary.property_value_summary,
            report.property_value_summary
        );
        assert_eq!(summary.uniform_summary, report.uniform_summary);
        assert_eq!(
            summary.standard_texture_slot_summary,
            report.standard_texture_slot_summary
        );
        assert_eq!(summary.texture_slot_summary, report.texture_slot_summary);

        let issue_state = report.issue_state();
        assert_eq!(issue_state.status(), RenderMaterialReadinessStatus::Invalid);
        assert!(!issue_state.is_ready());
        assert!(issue_state.uses_fallback());
        assert!(issue_state.has_diagnostics());
        assert_eq!(issue_state.validation_errors, report.validation_errors);
        assert_eq!(issue_state.fallback_usages, report.fallback_usages);
        assert_eq!(issue_state.diagnostics, report.diagnostics);

        let prepared_state = report.prepared_state();
        assert_eq!(
            prepared_state.property_value_summary,
            report.property_value_summary
        );
        assert_eq!(
            prepared_state.property_value_states,
            report.property_value_states
        );
        assert_eq!(prepared_state.uniform_summary, report.uniform_summary);
        assert_eq!(prepared_state.uniform_fields, report.uniform_fields);
        assert_eq!(
            prepared_state.uniform_unsupported,
            report.uniform_unsupported
        );
        assert_eq!(
            prepared_state.standard_texture_slot_summary,
            report.standard_texture_slot_summary
        );
        assert_eq!(
            prepared_state.standard_texture_slot_states,
            report.standard_texture_slot_states
        );
        assert_eq!(
            prepared_state.texture_slot_summary,
            report.texture_slot_summary
        );
        assert_eq!(
            prepared_state.non_standard_texture_slot_states,
            report.non_standard_texture_slot_states
        );

        let management_snapshot = report.management_snapshot();
        assert_eq!(management_snapshot.summary, summary);
        assert_eq!(management_snapshot.issue_state, issue_state);
        assert_eq!(management_snapshot.prepared_state, prepared_state);

        let material_id = ResourceId::from_stable_label("material:summary");
        let management_record = report.management_record(material_id);
        assert_eq!(management_record.material_id, material_id);
        assert_eq!(
            management_record.material_name.as_deref(),
            Some("SummaryMaterial")
        );
        assert_eq!(management_record.snapshot, management_snapshot);
        assert_eq!(
            management_record.status(),
            RenderMaterialReadinessStatus::Invalid
        );
        assert!(!management_record.is_ready());
        let management_overview_record = management_record.overview();
        assert_eq!(management_overview_record.material_id, material_id);
        assert_eq!(
            management_overview_record.material_name.as_deref(),
            Some("SummaryMaterial")
        );
        assert_eq!(management_overview_record.summary, summary);
        assert_eq!(
            management_overview_record.status(),
            RenderMaterialReadinessStatus::Invalid
        );
        assert!(!management_overview_record.is_ready());

        let ready_report = RenderMaterialReadinessReport {
            material_name: Some("ReadyMaterial".to_string()),
            dependencies: RenderMaterialDependencySet::new(AssetReference::from_locator(
                ResourceLocator::parse("res://shaders/ready_material.zshader")
                    .expect("valid shader locator"),
            )),
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
            validation_errors: Vec::new(),
            fallback_usages: Vec::new(),
            property_value_summary: None,
            property_value_states: Vec::new(),
            uniform_summary: None,
            uniform_fields: Vec::new(),
            uniform_unsupported: Vec::new(),
            standard_texture_slot_summary: None,
            standard_texture_slot_states: Vec::new(),
            texture_slot_summary: None,
            non_standard_texture_slot_states: Vec::new(),
            diagnostics: Vec::new(),
        };
        let records = vec![
            ready_report.management_record(ResourceId::from_stable_label("material:ready")),
            management_record,
        ];
        let record_summary = RenderMaterialManagementRecordSummary::from_records(&records);
        assert_eq!(
            record_summary.status,
            RenderMaterialReadinessStatus::Invalid
        );
        assert_eq!(record_summary.total_count, 2);
        assert_eq!(record_summary.ready_count, 1);
        assert_eq!(record_summary.diagnostic_count, 0);
        assert_eq!(record_summary.fallback_count, 0);
        assert_eq!(record_summary.invalid_count, 1);
        assert_eq!(record_summary.degraded_count(), 1);

        let record_set = RenderMaterialManagementRecordSet::from_records(records.clone());
        assert_eq!(record_set.summary, record_summary);
        assert_eq!(
            record_set.summary.status,
            RenderMaterialReadinessStatus::Invalid
        );
        assert_eq!(record_set.status_index.total_count(), 2);
        assert_eq!(record_set.status_index.degraded_count(), 1);
        assert_eq!(
            record_set
                .status_index
                .ids_for_status(RenderMaterialReadinessStatus::Ready),
            &[ResourceId::from_stable_label("material:ready")]
        );
        assert_eq!(
            record_set
                .status_index
                .ids_for_status(RenderMaterialReadinessStatus::Diagnostic),
            &[]
        );
        assert_eq!(
            record_set
                .status_index
                .ids_for_status(RenderMaterialReadinessStatus::Fallback),
            &[]
        );
        assert_eq!(
            record_set
                .status_index
                .ids_for_status(RenderMaterialReadinessStatus::Invalid),
            &[material_id]
        );
        assert_eq!(record_set.records, records);
        assert!(!record_set.is_empty());
        assert_eq!(record_set.len(), 2);
        let overview = record_set.overview();
        assert_eq!(
            overview,
            RenderMaterialManagementOverview::from_records(&records)
        );
        assert_eq!(
            overview,
            RenderMaterialManagementOverview::from_record_set(&record_set)
        );
        assert_eq!(overview.summary, record_summary);
        assert_eq!(
            overview.summary.status,
            RenderMaterialReadinessStatus::Invalid
        );
        assert_eq!(overview.status_index, record_set.status_index);
        assert_eq!(
            overview.status_index,
            RenderMaterialManagementStatusIndex::from_overview_records(&overview.records)
        );
        assert!(!overview.status_index.is_empty());
        assert_eq!(overview.status_index.total_count(), 2);
        assert_eq!(overview.status_index.degraded_count(), 1);
        assert!(!overview.is_empty());
        assert_eq!(overview.len(), 2);
        assert_eq!(
            overview.records,
            records
                .iter()
                .map(RenderMaterialManagementRecord::overview)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            overview.records[1].status(),
            RenderMaterialReadinessStatus::Invalid
        );
        assert!(!overview.records[1].is_ready());
        let ready_view = record_set.status_view(RenderMaterialReadinessStatus::Ready);
        assert_eq!(ready_view.status, RenderMaterialReadinessStatus::Ready);
        assert_eq!(
            ready_view.material_ids,
            vec![ResourceId::from_stable_label("material:ready")]
        );
        assert_eq!(ready_view.records, vec![records[0].overview()]);
        assert_eq!(ready_view.len(), 1);
        assert!(!ready_view.is_empty());
        assert_eq!(
            overview.status_view(RenderMaterialReadinessStatus::Ready),
            ready_view
        );
        assert_eq!(
            RenderMaterialManagementStatusView::from_records(
                &records,
                RenderMaterialReadinessStatus::Invalid,
            )
            .material_ids,
            vec![material_id]
        );
        assert!(record_set
            .status_view(RenderMaterialReadinessStatus::Fallback)
            .is_empty());

        let sorted_records = vec![
            report.management_record(ResourceId::from_stable_label("material:sort-z")),
            ready_report.management_record(ResourceId::from_stable_label("material:sort-a")),
        ];
        let name_sort = RenderMaterialManagementSortOrder::new(
            RenderMaterialManagementSortKey::MaterialName,
            RenderMaterialManagementSortDirection::Ascending,
        );
        let sorted_set = RenderMaterialManagementRecordSet::from_sorted_records(
            sorted_records.clone(),
            name_sort,
        );
        assert_eq!(
            sorted_set
                .records
                .iter()
                .map(|record| record.material_name.as_deref())
                .collect::<Vec<_>>(),
            vec![Some("ReadyMaterial"), Some("SummaryMaterial")]
        );
        assert_eq!(sorted_set.summary.total_count, 2);
        assert_eq!(sorted_set.status_index.total_count(), 2);
        assert_eq!(
            sorted_set
                .overview_sorted(RenderMaterialManagementSortOrder::new(
                    RenderMaterialManagementSortKey::Status,
                    RenderMaterialManagementSortDirection::Descending,
                ))
                .records[0]
                .status(),
            RenderMaterialReadinessStatus::Invalid
        );
        let sorted_ready_view = sorted_set.status_view_sorted(
            RenderMaterialReadinessStatus::Ready,
            RenderMaterialManagementSortOrder::new(
                RenderMaterialManagementSortKey::MaterialName,
                RenderMaterialManagementSortDirection::Descending,
            ),
        );
        assert_eq!(
            sorted_ready_view.records[0].material_name.as_deref(),
            Some("ReadyMaterial")
        );
        let sorted_query = sorted_set.query(
            RenderMaterialManagementQuery::new()
                .with_status(RenderMaterialReadinessStatus::Ready)
                .with_text_filter("ready")
                .with_page(RenderMaterialManagementPageRequest::new(0, Some(1))),
        );
        assert_eq!(sorted_query.summary.total_count, 1);
        assert_eq!(sorted_query.summary.ready_count, 1);
        assert_eq!(sorted_query.status_index.ready.len(), 1);
        assert_eq!(sorted_query.page.total_count, 1);
        assert_eq!(sorted_query.page.returned_count, 1);
        assert_eq!(
            sorted_query.records[0].material_name.as_deref(),
            Some("ReadyMaterial")
        );
    }
}
