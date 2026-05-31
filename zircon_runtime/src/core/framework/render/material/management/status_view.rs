use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialManagementOverview, RenderMaterialManagementOverviewRecord,
    RenderMaterialManagementRecord, RenderMaterialManagementRecordSet,
    RenderMaterialManagementSortOrder,
};
use crate::core::framework::render::material::readiness_report::RenderMaterialReadinessStatus;
use crate::core::resource::ResourceId;

/// Compact overview rows for one readiness status bucket.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialManagementStatusView {
    #[serde(default)]
    pub status: RenderMaterialReadinessStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub material_ids: Vec<ResourceId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub records: Vec<RenderMaterialManagementOverviewRecord>,
}

impl RenderMaterialManagementStatusView {
    pub fn from_records(
        records: &[RenderMaterialManagementRecord],
        status: RenderMaterialReadinessStatus,
    ) -> Self {
        let records = records
            .iter()
            .filter(|record| record.status() == status)
            .map(RenderMaterialManagementRecord::overview)
            .collect::<Vec<_>>();
        Self::from_overview_records(records, status)
    }

    pub fn from_records_sorted(
        records: &[RenderMaterialManagementRecord],
        status: RenderMaterialReadinessStatus,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> Self {
        Self::from_records(records, status).sorted(sort_order)
    }

    pub fn from_overview(
        overview: &RenderMaterialManagementOverview,
        status: RenderMaterialReadinessStatus,
    ) -> Self {
        let records = overview
            .records
            .iter()
            .filter(|record| record.status() == status)
            .cloned()
            .collect();
        Self::from_overview_records(records, status)
    }

    pub fn from_overview_sorted(
        overview: &RenderMaterialManagementOverview,
        status: RenderMaterialReadinessStatus,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> Self {
        Self::from_overview(overview, status).sorted(sort_order)
    }

    pub fn from_record_set(
        record_set: &RenderMaterialManagementRecordSet,
        status: RenderMaterialReadinessStatus,
    ) -> Self {
        Self::from_records(&record_set.records, status)
    }

    pub fn from_record_set_sorted(
        record_set: &RenderMaterialManagementRecordSet,
        status: RenderMaterialReadinessStatus,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> Self {
        Self::from_record_set(record_set, status).sorted(sort_order)
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn sorted(&self, sort_order: RenderMaterialManagementSortOrder) -> Self {
        let mut records = self.records.clone();
        sort_order.sort_overview_records(&mut records);
        Self::from_overview_records(records, self.status)
    }

    fn from_overview_records(
        records: Vec<RenderMaterialManagementOverviewRecord>,
        status: RenderMaterialReadinessStatus,
    ) -> Self {
        let material_ids = records.iter().map(|record| record.material_id).collect();
        Self {
            status,
            material_ids,
            records,
        }
    }
}
