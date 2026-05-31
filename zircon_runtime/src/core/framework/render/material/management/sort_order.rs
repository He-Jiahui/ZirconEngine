use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use super::{RenderMaterialManagementOverviewRecord, RenderMaterialManagementRecord};
use crate::core::framework::render::material::readiness_report::RenderMaterialReadinessStatus;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMaterialManagementSortKey {
    #[default]
    MaterialId,
    MaterialName,
    Status,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMaterialManagementSortDirection {
    #[default]
    Ascending,
    Descending,
}

/// Stable sort contract for derived material management rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialManagementSortOrder {
    #[serde(default)]
    pub key: RenderMaterialManagementSortKey,
    #[serde(default)]
    pub direction: RenderMaterialManagementSortDirection,
}

impl RenderMaterialManagementSortOrder {
    pub fn new(
        key: RenderMaterialManagementSortKey,
        direction: RenderMaterialManagementSortDirection,
    ) -> Self {
        Self { key, direction }
    }

    pub fn sort_records(&self, records: &mut [RenderMaterialManagementRecord]) {
        records.sort_by(|a, b| self.compare_records(a, b));
    }

    pub fn sort_overview_records(&self, records: &mut [RenderMaterialManagementOverviewRecord]) {
        records.sort_by(|a, b| self.compare_overview_records(a, b));
    }

    fn compare_records(
        &self,
        left: &RenderMaterialManagementRecord,
        right: &RenderMaterialManagementRecord,
    ) -> Ordering {
        let primary = match self.key {
            RenderMaterialManagementSortKey::MaterialId => {
                self.apply_direction(left.material_id.cmp(&right.material_id))
            }
            RenderMaterialManagementSortKey::MaterialName => compare_material_names_for_direction(
                &left.material_name,
                &right.material_name,
                self.direction,
            ),
            RenderMaterialManagementSortKey::Status => {
                self.apply_direction(status_rank(left.status()).cmp(&status_rank(right.status())))
            }
        };
        primary.then_with(|| compare_records_by_identity(left, right))
    }

    fn compare_overview_records(
        &self,
        left: &RenderMaterialManagementOverviewRecord,
        right: &RenderMaterialManagementOverviewRecord,
    ) -> Ordering {
        let primary = match self.key {
            RenderMaterialManagementSortKey::MaterialId => {
                self.apply_direction(left.material_id.cmp(&right.material_id))
            }
            RenderMaterialManagementSortKey::MaterialName => compare_material_names_for_direction(
                &left.material_name,
                &right.material_name,
                self.direction,
            ),
            RenderMaterialManagementSortKey::Status => {
                self.apply_direction(status_rank(left.status()).cmp(&status_rank(right.status())))
            }
        };
        primary.then_with(|| compare_overview_records_by_identity(left, right))
    }

    fn apply_direction(&self, ordering: Ordering) -> Ordering {
        match self.direction {
            RenderMaterialManagementSortDirection::Ascending => ordering,
            RenderMaterialManagementSortDirection::Descending => ordering.reverse(),
        }
    }
}

fn compare_records_by_identity(
    left: &RenderMaterialManagementRecord,
    right: &RenderMaterialManagementRecord,
) -> Ordering {
    compare_material_names(&left.material_name, &right.material_name)
        .then_with(|| left.material_id.cmp(&right.material_id))
}

fn compare_overview_records_by_identity(
    left: &RenderMaterialManagementOverviewRecord,
    right: &RenderMaterialManagementOverviewRecord,
) -> Ordering {
    compare_material_names(&left.material_name, &right.material_name)
        .then_with(|| left.material_id.cmp(&right.material_id))
}

fn compare_material_names(left: &Option<String>, right: &Option<String>) -> Ordering {
    match (left.as_deref(), right.as_deref()) {
        (Some(left), Some(right)) => compare_material_name_text(left, right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_material_names_for_direction(
    left: &Option<String>,
    right: &Option<String>,
    direction: RenderMaterialManagementSortDirection,
) -> Ordering {
    match (left.as_deref(), right.as_deref()) {
        (Some(left), Some(right)) => match direction {
            RenderMaterialManagementSortDirection::Ascending => {
                compare_material_name_text(left, right)
            }
            RenderMaterialManagementSortDirection::Descending => {
                compare_material_name_text(left, right).reverse()
            }
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_material_name_text(left: &str, right: &str) -> Ordering {
    left.to_ascii_lowercase()
        .cmp(&right.to_ascii_lowercase())
        .then_with(|| left.cmp(right))
}

fn status_rank(status: RenderMaterialReadinessStatus) -> u8 {
    match status {
        RenderMaterialReadinessStatus::Ready => 0,
        RenderMaterialReadinessStatus::Diagnostic => 1,
        RenderMaterialReadinessStatus::Fallback => 2,
        RenderMaterialReadinessStatus::Invalid => 3,
    }
}
