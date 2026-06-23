use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    binding::{UiBindingUpdateReport, UiEventKind},
    component::{UiComponentEvent, UiValue},
    dispatch::{UiPointerComponentEvent, UiPointerComponentEventReason},
    event_ui::UiNodeId,
    surface::{UiPointerActivationPhase, UiPointerEventKind, UiPointerRoute},
    tree::{UiTemplateNodeMetadata, UiTreeError},
};

use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};

mod columns;
mod selection;
mod virtualization;

const TABLE_OWNER_COMPONENTS: [&str; 2] = ["Table", "DataGrid"];
const TABLE_OWNER_ROLES: [&str; 3] = ["table", "data-grid", "mui-x-data-grid"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::surface::surface) struct UiDefaultTablePointerActionReport {
    pub handled_by: Option<UiNodeId>,
    pub captured_by: Option<UiNodeId>,
    pub released_capture: Option<UiNodeId>,
    pub damage_node: Option<UiNodeId>,
}

struct UiDefaultTableColumnResizeStart {
    owner_id: UiNodeId,
    field: String,
    start_width: f64,
    min_width: f64,
}

struct UiDefaultTableSortHeader {
    owner_id: UiNodeId,
    field: String,
    direction: &'static str,
}

struct UiTableColumnResizeDragToken {
    field: String,
    start_width: f64,
    min_width: f64,
}

impl UiSurface {
    pub(in crate::ui::surface::surface) fn apply_default_table_pointer_action(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<UiDefaultTablePointerActionReport, UiTreeError> {
        match route.activation_phase {
            UiPointerActivationPhase::PrimaryPress => {
                self.apply_default_table_column_resize_press(route, events)
            }
            UiPointerActivationPhase::Hover if matches!(route.kind, UiPointerEventKind::Move) => {
                self.apply_default_table_column_resize_drag(route, events, binding_reports)
            }
            UiPointerActivationPhase::PrimaryRelease => {
                let resize =
                    self.apply_default_table_column_resize_release(route, events, binding_reports)?;
                if resize.handled_by.is_some() {
                    Ok(resize)
                } else {
                    let sort = self.apply_default_table_sort_header_release(
                        route,
                        events,
                        binding_reports,
                    )?;
                    if sort.handled_by.is_some() {
                        Ok(sort)
                    } else {
                        self.apply_default_table_row_selection_release(
                            route,
                            events,
                            binding_reports,
                        )
                    }
                }
            }
            UiPointerActivationPhase::Scroll
                if matches!(route.kind, UiPointerEventKind::Scroll) =>
            {
                self.apply_default_table_virtual_scroll(route, events, binding_reports)
            }
            _ => Ok(UiDefaultTablePointerActionReport::default()),
        }
    }

    fn apply_default_table_column_resize_press(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
    ) -> Result<UiDefaultTablePointerActionReport, UiTreeError> {
        let Some(start) = self.default_table_column_resize_start(route)? else {
            return Ok(UiDefaultTablePointerActionReport::default());
        };
        self.capture_pointer(start.owner_id)?;
        let drag = self.input.begin_pointer_drag_with_property(
            start.owner_id,
            route.point,
            Some(columns::encode_table_column_resize_drag(
                start.start_width,
                start.min_width,
                &start.field,
            )),
        );
        self.push_pointer_component_events_with_drag_metrics(
            events,
            start.owner_id,
            UiEventKind::DragBegin,
            UiComponentEvent::BeginDrag {
                property: "column_width".to_string(),
            },
            UiPointerComponentEventReason::PressBegin,
            Some(drag),
        )?;
        Ok(UiDefaultTablePointerActionReport {
            handled_by: Some(start.owner_id),
            captured_by: Some(start.owner_id),
            damage_node: Some(start.owner_id),
            ..UiDefaultTablePointerActionReport::default()
        })
    }

    fn apply_default_table_column_resize_drag(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<UiDefaultTablePointerActionReport, UiTreeError> {
        let Some(owner_id) = route.captured else {
            return Ok(UiDefaultTablePointerActionReport::default());
        };
        let Some(token) = self
            .input
            .pointer_drag_property(owner_id)
            .and_then(columns::decode_table_column_resize_drag)
        else {
            return Ok(UiDefaultTablePointerActionReport::default());
        };
        let drag = self.input.update_pointer_drag(owner_id, route.point);
        let next_width = (token.start_width + f64::from(drag.delta.x)).max(token.min_width);
        if let Some(delta) = self.apply_default_table_column_width(
            owner_id,
            &token.field,
            next_width,
            events,
            binding_reports,
            UiPointerComponentEventReason::DirectBinding,
        )? {
            self.push_pointer_component_events_with_drag_metrics(
                events,
                owner_id,
                UiEventKind::DragUpdate,
                UiComponentEvent::DragDelta {
                    property: "column_width".to_string(),
                    delta,
                },
                UiPointerComponentEventReason::DirectBinding,
                Some(drag),
            )?;
        }
        Ok(UiDefaultTablePointerActionReport {
            handled_by: Some(owner_id),
            damage_node: Some(owner_id),
            ..UiDefaultTablePointerActionReport::default()
        })
    }

    fn apply_default_table_column_resize_release(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<UiDefaultTablePointerActionReport, UiTreeError> {
        let Some(owner_id) = route.captured.or(route.pressed) else {
            return Ok(UiDefaultTablePointerActionReport::default());
        };
        let Some(token) = self
            .input
            .pointer_drag_property(owner_id)
            .and_then(columns::decode_table_column_resize_drag)
        else {
            return Ok(UiDefaultTablePointerActionReport::default());
        };
        let drag = self.input.end_pointer_drag(owner_id, route.point);
        let next_width = (token.start_width + f64::from(drag.delta.x)).max(token.min_width);
        let _ = self.apply_default_table_column_width(
            owner_id,
            &token.field,
            next_width,
            events,
            binding_reports,
            UiPointerComponentEventReason::DefaultClick,
        )?;
        self.push_pointer_component_events_with_drag_metrics(
            events,
            owner_id,
            UiEventKind::DragEnd,
            UiComponentEvent::EndDrag {
                property: "column_width".to_string(),
            },
            UiPointerComponentEventReason::PressEnd,
            Some(drag),
        )?;
        Ok(UiDefaultTablePointerActionReport {
            handled_by: Some(owner_id),
            released_capture: Some(owner_id),
            damage_node: Some(owner_id),
            ..UiDefaultTablePointerActionReport::default()
        })
    }

    fn apply_default_table_sort_header_release(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<UiDefaultTablePointerActionReport, UiTreeError> {
        let Some(sort) = self.default_table_sort_header(route)? else {
            return Ok(UiDefaultTablePointerActionReport::default());
        };
        if !self.apply_default_table_sort(&sort, events, binding_reports)? {
            return Ok(UiDefaultTablePointerActionReport::default());
        }
        Ok(UiDefaultTablePointerActionReport {
            handled_by: Some(sort.owner_id),
            damage_node: Some(sort.owner_id),
            ..UiDefaultTablePointerActionReport::default()
        })
    }

    fn apply_default_table_sort(
        &mut self,
        sort: &UiDefaultTableSortHeader,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let (write_sort_model, client_sorting) = {
            let metadata = self.template_metadata(sort.owner_id)?;
            (
                is_data_grid_owner(metadata) || metadata.attributes.contains_key("sortModel"),
                columns::table_uses_client_sorting(metadata),
            )
        };

        let mut changed = false;
        changed |= self.apply_table_mutation(
            sort.owner_id,
            "sort_column",
            UiValue::String(sort.field.clone()),
            binding_reports,
        )?;
        changed |= self.apply_table_mutation(
            sort.owner_id,
            "sort_direction",
            UiValue::String(sort.direction.to_string()),
            binding_reports,
        )?;
        if write_sort_model {
            changed |= self.apply_table_sort_model_mutation(
                sort.owner_id,
                &sort.field,
                sort.direction,
                binding_reports,
            )?;
        }
        changed |= self.apply_table_columns_sort_direction_mutation(
            sort.owner_id,
            &sort.field,
            sort.direction,
            binding_reports,
        )?;
        if client_sorting {
            changed |= self.apply_table_rows_sort_mutation(
                sort.owner_id,
                &sort.field,
                sort.direction,
                binding_reports,
            )?;
        }
        if !changed {
            return Ok(false);
        }

        self.push_pointer_component_events(
            events,
            sort.owner_id,
            UiEventKind::Change,
            UiComponentEvent::ValueChanged {
                property: "sort_column".to_string(),
                value: UiValue::String(sort.field.clone()),
            },
            UiPointerComponentEventReason::DefaultClick,
        )?;
        Ok(true)
    }

    fn default_table_sort_header(
        &self,
        route: &UiPointerRoute,
    ) -> Result<Option<UiDefaultTableSortHeader>, UiTreeError> {
        if route.captured.is_some() {
            return Ok(None);
        }
        let Some(header_id) = route.bubbled.iter().copied().find(|node_id| {
            self.tree
                .node(*node_id)
                .and_then(|node| node.template_metadata.as_ref())
                .is_some_and(columns::is_table_column_sort_header)
        }) else {
            return Ok(None);
        };
        let Some(header_metadata) = self
            .tree
            .node(header_id)
            .and_then(|node| node.template_metadata.as_ref())
        else {
            return Ok(None);
        };
        let Some(field) = columns::table_column_field(header_metadata) else {
            return Ok(None);
        };
        let Some(owner_id) = route.bubbled.iter().copied().find(|node_id| {
            *node_id != header_id
                && self
                    .tree
                    .node(*node_id)
                    .and_then(|node| node.template_metadata.as_ref())
                    .is_some_and(is_table_owner)
        }) else {
            return Ok(None);
        };
        let Some(owner) = self.tree.node(owner_id) else {
            return Ok(None);
        };
        let Some(owner_metadata) = owner.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(owner_id, owner, owner_metadata)
            || columns::table_sorting_disabled(owner_metadata)
            || columns::table_column_sorting_disabled(header_metadata, owner_metadata, &field)
        {
            return Ok(None);
        }
        Ok(Some(UiDefaultTableSortHeader {
            owner_id,
            direction: columns::next_table_sort_direction(owner_metadata, &field),
            field,
        }))
    }

    fn default_table_column_resize_start(
        &self,
        route: &UiPointerRoute,
    ) -> Result<Option<UiDefaultTableColumnResizeStart>, UiTreeError> {
        let Some(handle_id) = route.bubbled.iter().copied().find(|node_id| {
            self.tree
                .node(*node_id)
                .and_then(|node| node.template_metadata.as_ref())
                .is_some_and(columns::is_table_column_resize_handle)
        }) else {
            return Ok(None);
        };
        let Some(handle_metadata) = self
            .tree
            .node(handle_id)
            .and_then(|node| node.template_metadata.as_ref())
        else {
            return Ok(None);
        };
        let Some(field) = columns::table_column_field(handle_metadata) else {
            return Ok(None);
        };
        let Some(owner_id) = route.bubbled.iter().copied().find(|node_id| {
            *node_id != handle_id
                && self
                    .tree
                    .node(*node_id)
                    .and_then(|node| node.template_metadata.as_ref())
                    .is_some_and(is_table_owner)
        }) else {
            return Ok(None);
        };
        let Some(owner) = self.tree.node(owner_id) else {
            return Ok(None);
        };
        let Some(owner_metadata) = owner.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(owner_id, owner, owner_metadata)
            || columns::table_column_resize_disabled(owner_metadata)
        {
            return Ok(None);
        }
        let Some(start_width) = columns::table_column_width(owner_metadata, &field) else {
            return Ok(None);
        };
        let min_width = columns::table_min_column_width(owner_metadata, &field);
        Ok(Some(UiDefaultTableColumnResizeStart {
            owner_id,
            field,
            start_width,
            min_width,
        }))
    }

    fn apply_default_table_column_width(
        &mut self,
        owner_id: UiNodeId,
        field: &str,
        width: f64,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
        reason: UiPointerComponentEventReason,
    ) -> Result<Option<f64>, UiTreeError> {
        let previous_width = self
            .template_metadata(owner_id)
            .ok()
            .and_then(|metadata| columns::table_column_width(metadata, field))
            .unwrap_or(width);
        let mut changed = false;
        changed |=
            self.apply_table_column_widths_mutation(owner_id, field, width, binding_reports)?;
        changed |=
            self.apply_table_columns_width_mutation(owner_id, field, width, binding_reports)?;
        if !changed {
            return Ok(None);
        }

        self.push_pointer_component_events(
            events,
            owner_id,
            UiEventKind::Change,
            UiComponentEvent::ValueChanged {
                property: "column_width".to_string(),
                value: columns::column_width_payload(field, width),
            },
            reason,
        )?;
        Ok(Some(width - previous_width))
    }

    fn apply_table_column_widths_mutation(
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

    fn apply_table_columns_width_mutation(
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

    fn apply_table_sort_model_mutation(
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

    fn apply_table_columns_sort_direction_mutation(
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

    fn apply_table_rows_sort_mutation(
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

    fn apply_table_mutation(
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

pub(super) fn is_default_table_behavior(metadata: &UiTemplateNodeMetadata) -> bool {
    is_table_owner(metadata)
        || columns::is_table_column_resize_handle(metadata)
        || columns::is_table_column_sort_header(metadata)
        || selection::is_table_row(metadata)
}

fn is_table_owner(metadata: &UiTemplateNodeMetadata) -> bool {
    TABLE_OWNER_COMPONENTS.contains(&metadata.component.as_str())
        || role_is_one_of(metadata, &TABLE_OWNER_ROLES)
}

fn is_data_grid_owner(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.component == "DataGrid" || role_is_one_of(metadata, &["data-grid", "mui-x-data-grid"])
}

fn role_is_one_of(metadata: &UiTemplateNodeMetadata, roles: &[&str]) -> bool {
    string_attribute(metadata, "role").is_some_and(|role| roles.contains(&role.as_str()))
}

fn string_attribute(metadata: &UiTemplateNodeMetadata, property: &str) -> Option<String> {
    metadata
        .attributes
        .get(property)
        .and_then(toml::Value::as_str)
        .map(str::to_string)
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, property: &str) -> bool {
    metadata
        .attributes
        .get(property)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn explicit_false_attribute(metadata: &UiTemplateNodeMetadata, property: &str) -> bool {
    metadata
        .attributes
        .get(property)
        .and_then(toml::Value::as_bool)
        == Some(false)
}

fn toml_number(value: &toml::Value) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
}
