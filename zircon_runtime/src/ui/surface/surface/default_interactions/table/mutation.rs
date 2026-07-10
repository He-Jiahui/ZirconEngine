use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    binding::UiBindingUpdateReport, component::UiValue, event_ui::UiNodeId, tree::UiTreeError,
};

use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};

use super::columns;

impl UiSurface {
    pub(super) fn apply_table_column_widths_mutation(
        &mut self,
        owner_id: UiNodeId,
        field: &str,
        width: f64,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let mut widths = self
            .template_metadata(owner_id)
            .ok()
            .and_then(|metadata| metadata.attributes.get("column_widths"))
            .map(UiValue::from_toml)
            .and_then(|value| match value {
                UiValue::Map(values) => Some(values),
                _ => None,
            })
            .unwrap_or_default();
        widths.insert(field.to_string(), UiValue::Float(width));
        self.apply_table_mutation(
            owner_id,
            "column_widths",
            UiValue::Map(widths),
            binding_reports,
        )
    }

    pub(super) fn apply_table_columns_width_mutation(
        &mut self,
        owner_id: UiNodeId,
        field: &str,
        width: f64,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let Some(mut columns) = self
            .template_metadata(owner_id)
            .ok()
            .and_then(|metadata| metadata.attributes.get("columns"))
            .map(UiValue::from_toml)
            .and_then(|value| match value {
                UiValue::Array(columns) => Some(columns),
                _ => None,
            })
        else {
            return Ok(false);
        };

        let mut found = false;
        for column in &mut columns {
            let UiValue::Map(values) = column else {
                continue;
            };
            if columns::table_column_matches(values, field) {
                values.insert("width".to_string(), UiValue::Float(width));
                found = true;
                break;
            }
        }
        if !found {
            return Ok(false);
        }

        self.apply_table_mutation(
            owner_id,
            "columns",
            UiValue::Array(columns),
            binding_reports,
        )
    }

    pub(super) fn apply_table_sort_model_mutation(
        &mut self,
        owner_id: UiNodeId,
        field: &str,
        direction: &str,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        self.apply_table_mutation(
            owner_id,
            "sortModel",
            UiValue::Array(vec![UiValue::Map(BTreeMap::from([
                ("field".to_string(), UiValue::String(field.to_string())),
                ("sort".to_string(), UiValue::String(direction.to_string())),
            ]))]),
            binding_reports,
        )
    }

    pub(super) fn apply_table_columns_sort_direction_mutation(
        &mut self,
        owner_id: UiNodeId,
        field: &str,
        direction: &str,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let Some(mut columns) = self
            .template_metadata(owner_id)
            .ok()
            .and_then(|metadata| metadata.attributes.get("columns"))
            .map(UiValue::from_toml)
            .and_then(|value| match value {
                UiValue::Array(columns) => Some(columns),
                _ => None,
            })
        else {
            return Ok(false);
        };

        let mut found = false;
        for column in &mut columns {
            let UiValue::Map(values) = column else {
                continue;
            };
            let next_direction = if columns::table_column_matches(values, field) {
                found = true;
                direction
            } else {
                "none"
            };
            values.insert(
                "sortDirection".to_string(),
                UiValue::String(next_direction.to_string()),
            );
        }
        if !found {
            return Ok(false);
        }

        self.apply_table_mutation(
            owner_id,
            "columns",
            UiValue::Array(columns),
            binding_reports,
        )
    }

    pub(super) fn apply_table_rows_sort_mutation(
        &mut self,
        owner_id: UiNodeId,
        field: &str,
        direction: &str,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let Some(mut rows) = self
            .template_metadata(owner_id)
            .ok()
            .and_then(|metadata| metadata.attributes.get("rows"))
            .map(UiValue::from_toml)
            .and_then(|value| match value {
                UiValue::Array(rows) => Some(rows),
                _ => None,
            })
        else {
            return Ok(false);
        };

        rows.sort_by(|left, right| columns::compare_table_row_value(left, right, field));
        if direction == "desc" {
            rows.reverse();
        }
        self.apply_table_mutation(owner_id, "rows", UiValue::Array(rows), binding_reports)
    }

    pub(super) fn apply_table_mutation(
        &mut self,
        owner_id: UiNodeId,
        property: impl Into<String>,
        value: UiValue,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
            owner_id, property, value,
        ))?;
        if matches!(report.status, UiPropertyMutationStatus::Accepted) {
            binding_reports.push(report.binding);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
