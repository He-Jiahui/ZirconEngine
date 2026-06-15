use zircon_runtime_interface::ui::{
    binding::{UiBindingUpdateReport, UiEventKind},
    component::{UiComponentEvent, UiValue},
    dispatch::{UiPointerComponentEvent, UiPointerComponentEventReason},
    event_ui::UiNodeId,
    surface::UiPointerRoute,
    tree::{UiTemplateNodeMetadata, UiTreeError},
};

use crate::ui::surface::UiSurface;

const TABLE_ROW_COMPONENTS: [&str; 5] =
    ["TableRow", "DataGridRow", "TableBodyRow", "DataRow", "Row"];
const TABLE_ROW_ROLES: [&str; 4] = ["table-row", "data-grid-row", "data-row", "row"];
const TABLE_ROW_ID_PROPERTIES: [&str; 8] = [
    "row_id",
    "rowId",
    "option_id",
    "optionId",
    "id",
    "key",
    "value",
    "name",
];
const TABLE_ROW_INDEX_PROPERTIES: [&str; 4] = ["row_index", "rowIndex", "index", "visible_index"];

struct UiDefaultTableRowSelection {
    owner_id: UiNodeId,
    property: &'static str,
    row_id: String,
    row_index: usize,
    write_value: bool,
}

struct TableRowHit {
    owner_id: UiNodeId,
    row_id: String,
    row_index: usize,
}

impl UiSurface {
    pub(super) fn apply_default_table_row_selection_release(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<super::UiDefaultTablePointerActionReport, UiTreeError> {
        let Some(selection) = self.default_table_row_selection(route)? else {
            return Ok(super::UiDefaultTablePointerActionReport::default());
        };
        if !self.apply_table_row_selection(&selection, events, binding_reports)? {
            return Ok(super::UiDefaultTablePointerActionReport::default());
        }
        Ok(super::UiDefaultTablePointerActionReport {
            handled_by: Some(selection.owner_id),
            damage_node: Some(selection.owner_id),
            ..super::UiDefaultTablePointerActionReport::default()
        })
    }

    fn default_table_row_selection(
        &self,
        route: &UiPointerRoute,
    ) -> Result<Option<UiDefaultTableRowSelection>, UiTreeError> {
        if route.captured.is_some() || route.click_target.is_none() {
            return Ok(None);
        }
        let Some(hit) = self.table_row_hit(route)? else {
            return Ok(None);
        };
        let owner = self
            .tree
            .node(hit.owner_id)
            .ok_or(UiTreeError::MissingNode(hit.owner_id))?;
        let Some(owner_metadata) = owner.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(hit.owner_id, owner, owner_metadata)
            || table_row_selection_disabled(owner_metadata)
            || table_option_is_disabled(owner_metadata, &hit.row_id)
        {
            return Ok(None);
        }

        let property = table_row_selection_property(owner_metadata);
        Ok(Some(UiDefaultTableRowSelection {
            owner_id: hit.owner_id,
            property,
            row_id: hit.row_id,
            row_index: hit.row_index,
            write_value: !super::is_data_grid_owner(owner_metadata),
        }))
    }

    fn table_row_hit(&self, route: &UiPointerRoute) -> Result<Option<TableRowHit>, UiTreeError> {
        let mut row_seen = false;
        let mut row_id = None;
        let mut row_index = None;
        let mut blocked = false;
        let hit_route = if route.stacked.is_empty() {
            route.bubbled.as_slice()
        } else {
            route.stacked.as_slice()
        };

        for node_id in hit_route {
            let node = self
                .tree
                .node(*node_id)
                .ok_or(UiTreeError::MissingNode(*node_id))?;
            if !self.node_interaction_enabled(*node_id)? {
                blocked = true;
            }
            let Some(metadata) = node.template_metadata.as_ref() else {
                continue;
            };
            if is_table_row(metadata) {
                row_seen = true;
                row_id = row_id.or_else(|| table_row_id(metadata));
                row_index = row_index.or_else(|| table_row_index(metadata));
                blocked |= table_row_disabled(metadata);
            }
            if super::is_table_owner(metadata) {
                if blocked || !row_seen {
                    return Ok(None);
                }
                let Some(row_id) = row_id.or_else(|| {
                    row_index.and_then(|index| table_row_id_from_owner(metadata, index))
                }) else {
                    return Ok(None);
                };
                let row_index = row_index
                    .or_else(|| table_row_index_from_owner(metadata, &row_id))
                    .unwrap_or_default();
                return Ok(Some(TableRowHit {
                    owner_id: *node_id,
                    row_id,
                    row_index,
                }));
            }
        }
        Ok(None)
    }

    fn apply_table_row_selection(
        &mut self,
        selection: &UiDefaultTableRowSelection,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let mut changed = false;
        changed |= self.apply_table_mutation(
            selection.owner_id,
            selection.property,
            string_array_value(&[selection.row_id.clone()]),
            binding_reports,
        )?;
        changed |= self.apply_table_mutation(
            selection.owner_id,
            "focused_index",
            UiValue::Int(selection.row_index as i64),
            binding_reports,
        )?;
        changed |= self.apply_table_mutation(
            selection.owner_id,
            "selected_index",
            UiValue::Int(selection.row_index as i64),
            binding_reports,
        )?;
        if selection.write_value {
            changed |= self.apply_table_mutation(
                selection.owner_id,
                "value",
                UiValue::String(selection.row_id.clone()),
                binding_reports,
            )?;
        }
        if !changed {
            return Ok(false);
        }

        self.push_pointer_component_events(
            events,
            selection.owner_id,
            UiEventKind::Change,
            UiComponentEvent::SelectOption {
                property: selection.property.to_string(),
                option_id: selection.row_id.clone(),
                selected: true,
            },
            UiPointerComponentEventReason::DefaultClick,
        )?;
        Ok(true)
    }
}

pub(super) fn is_table_row(metadata: &UiTemplateNodeMetadata) -> bool {
    TABLE_ROW_COMPONENTS.contains(&metadata.component.as_str())
        || super::role_is_one_of(metadata, &TABLE_ROW_ROLES)
}

fn table_row_selection_property(metadata: &UiTemplateNodeMetadata) -> &'static str {
    if super::is_data_grid_owner(metadata) || metadata.attributes.contains_key("rowSelectionModel")
    {
        "rowSelectionModel"
    } else if metadata.attributes.contains_key("selectedRows") {
        "selectedRows"
    } else {
        "selected_rows"
    }
}

fn table_row_selection_disabled(metadata: &UiTemplateNodeMetadata) -> bool {
    super::bool_attribute(metadata, "disableRowSelectionOnClick")
        || super::bool_attribute(metadata, "disable_row_selection_on_click")
        || super::explicit_false_attribute(metadata, "rowSelection")
        || super::explicit_false_attribute(metadata, "row_selection")
        || super::explicit_false_attribute(metadata, "selectableRows")
        || super::explicit_false_attribute(metadata, "selectable_rows")
}

fn table_row_disabled(metadata: &UiTemplateNodeMetadata) -> bool {
    super::bool_attribute(metadata, "disabled")
        || super::explicit_false_attribute(metadata, "enabled")
}

fn table_row_id(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    TABLE_ROW_ID_PROPERTIES
        .iter()
        .find_map(|property| super::string_attribute(metadata, property))
        .filter(|id| !id.is_empty())
}

fn table_row_index(metadata: &UiTemplateNodeMetadata) -> Option<usize> {
    TABLE_ROW_INDEX_PROPERTIES
        .iter()
        .find_map(|property| metadata.attributes.get(*property)?.as_integer())
        .and_then(|index| usize::try_from(index).ok())
}

fn table_option_is_disabled(metadata: &UiTemplateNodeMetadata, row_id: &str) -> bool {
    metadata
        .attributes
        .get("disabled_options")
        .or_else(|| metadata.attributes.get("disabledRows"))
        .or_else(|| metadata.attributes.get("disabled_rows"))
        .is_some_and(|value| value_contains_row_id(value, row_id))
}

fn table_row_id_from_owner(metadata: &UiTemplateNodeMetadata, index: usize) -> Option<String> {
    metadata
        .attributes
        .get("rows")
        .and_then(toml::Value::as_array)
        .and_then(|rows| rows.get(index))
        .and_then(toml_row_id)
}

fn table_row_index_from_owner(metadata: &UiTemplateNodeMetadata, row_id: &str) -> Option<usize> {
    metadata
        .attributes
        .get("rows")
        .and_then(toml::Value::as_array)?
        .iter()
        .position(|row| toml_row_id(row).as_deref() == Some(row_id))
}

fn toml_row_id(value: &toml::Value) -> Option<String> {
    match value {
        toml::Value::String(value) if !value.is_empty() => Some(value.clone()),
        toml::Value::Table(values) => TABLE_ROW_ID_PROPERTIES
            .iter()
            .find_map(|property| values.get(*property)?.as_str())
            .filter(|id| !id.is_empty())
            .map(str::to_string),
        _ => None,
    }
}

fn value_contains_row_id(value: &toml::Value, row_id: &str) -> bool {
    match value {
        toml::Value::Array(values) => values
            .iter()
            .any(|value| value_contains_row_id(value, row_id)),
        toml::Value::String(value) => value == row_id,
        toml::Value::Table(values) => TABLE_ROW_ID_PROPERTIES
            .iter()
            .filter_map(|property| values.get(*property))
            .any(|value| value_contains_row_id(value, row_id)),
        _ => false,
    }
}

fn string_array_value(values: &[String]) -> UiValue {
    UiValue::Array(values.iter().cloned().map(UiValue::String).collect())
}
