use serde::{Deserialize, Serialize};

use super::{RenderMaterialManagementOverviewRecord, RenderMaterialManagementRecord};
use crate::core::framework::render::material::readiness_report::RenderMaterialReadinessStatus;
use crate::core::resource::ResourceId;

/// Material ids bucketed by readiness status for panel filters and badges.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialManagementStatusIndex {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ready: Vec<ResourceId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostic: Vec<ResourceId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallback: Vec<ResourceId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invalid: Vec<ResourceId>,
}

impl RenderMaterialManagementStatusIndex {
    pub fn from_records(records: &[RenderMaterialManagementRecord]) -> Self {
        let mut index = Self::default();
        for record in records {
            index.push(record.material_id, record.status());
        }
        index
    }

    pub fn from_overview_records(records: &[RenderMaterialManagementOverviewRecord]) -> Self {
        let mut index = Self::default();
        for record in records {
            index.push(record.material_id, record.status());
        }
        index
    }

    pub fn push(&mut self, material_id: ResourceId, status: RenderMaterialReadinessStatus) {
        match status {
            RenderMaterialReadinessStatus::Ready => self.ready.push(material_id),
            RenderMaterialReadinessStatus::Diagnostic => self.diagnostic.push(material_id),
            RenderMaterialReadinessStatus::Fallback => self.fallback.push(material_id),
            RenderMaterialReadinessStatus::Invalid => self.invalid.push(material_id),
        }
    }

    pub fn ids_for_status(&self, status: RenderMaterialReadinessStatus) -> &[ResourceId] {
        match status {
            RenderMaterialReadinessStatus::Ready => &self.ready,
            RenderMaterialReadinessStatus::Diagnostic => &self.diagnostic,
            RenderMaterialReadinessStatus::Fallback => &self.fallback,
            RenderMaterialReadinessStatus::Invalid => &self.invalid,
        }
    }

    pub fn total_count(&self) -> usize {
        self.ready.len() + self.diagnostic.len() + self.fallback.len() + self.invalid.len()
    }

    pub fn degraded_count(&self) -> usize {
        self.diagnostic.len() + self.fallback.len() + self.invalid.len()
    }

    pub fn is_empty(&self) -> bool {
        self.total_count() == 0
    }
}
