use serde::{Deserialize, Serialize};

use super::RenderMaterialManagementOverviewRecord;
use crate::core::framework::render::material::readiness_report::{
    RenderMaterialIssueState, RenderMaterialPreparedState, RenderMaterialReadinessStatus,
    RenderMaterialReadinessSummary,
};
use crate::core::resource::ResourceId;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialManagementSnapshot {
    pub summary: RenderMaterialReadinessSummary,
    #[serde(default)]
    pub issue_state: RenderMaterialIssueState,
    #[serde(default)]
    pub prepared_state: RenderMaterialPreparedState,
}

/// Resource-keyed row for material management panels and debug snapshots.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialManagementRecord {
    pub material_id: ResourceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_name: Option<String>,
    #[serde(default)]
    pub snapshot: RenderMaterialManagementSnapshot,
}

impl RenderMaterialManagementRecord {
    pub fn status(&self) -> RenderMaterialReadinessStatus {
        self.snapshot.summary.status
    }

    pub fn is_ready(&self) -> bool {
        self.snapshot.summary.is_ready
    }

    pub fn overview(&self) -> RenderMaterialManagementOverviewRecord {
        RenderMaterialManagementOverviewRecord {
            material_id: self.material_id,
            material_name: self.material_name.clone(),
            summary: self.snapshot.summary,
        }
    }
}
