use std::collections::BTreeMap;

use crate::ui::UiDesignerSelectionModel;
use toml::{map::Map, Value};
use zircon_ui::template::UiChildMount;
use zircon_ui::UiAssetDocument;
use zircon_ui::template::UiNodeDefinition;

use super::style_rule_declarations::parse_declaration_literal;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiAssetInspectorSemanticEntry {
    pub path: String,
    pub literal: String,
}

impl UiAssetInspectorSemanticEntry {
    pub fn label(&self) -> String {
        format!("{} = {}", self.path, self.literal)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiAssetInspectorSemanticGroup {
    pub title: String,
    pub entries: Vec<UiAssetInspectorSemanticEntry>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiAssetStructuredSlotSemanticFields {
    pub kind: String,
    pub linear_main_weight: String,
    pub linear_main_stretch: String,
    pub linear_cross_weight: String,
    pub linear_cross_stretch: String,
    pub overlay_anchor_x: String,
    pub overlay_anchor_y: String,
    pub overlay_pivot_x: String,
    pub overlay_pivot_y: String,
    pub overlay_position_x: String,
    pub overlay_position_y: String,
    pub overlay_z_index: String,
    pub grid_row: String,
    pub grid_column: String,
    pub grid_row_span: String,
    pub grid_column_span: String,
    pub flow_break_before: String,
    pub flow_alignment: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiAssetStructuredLayoutSemanticFields {
    pub kind: String,
    pub box_gap: String,
    pub scroll_axis: String,
    pub scroll_gap: String,
    pub scrollbar_visibility: String,
    pub virtualization_item_extent: String,
    pub virtualization_overscan: String,
    pub clip: String,
}

pub(crate) fn build_slot_semantic_group(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> UiAssetInspectorSemanticGroup {
    let Some(child_mount) = selected_child_mount(document, selection) else {
        return UiAssetInspectorSemanticGroup::default();
    };
    let Some(parent) = selected_parent_node(document, selection) else {
        return UiAssetInspectorSemanticGroup::default();
    };
    match semantic_parent_kind(parent).as_deref() {
        Some("HorizontalBox") | Some("VerticalBox") => {
            semantic_group("Linear Slot", &child_mount.slot, LINEAR_SLOT_PATHS)
        }
        Some("Overlay") => semantic_group("Overlay Slot", &child_mount.slot, OVERLAY_SLOT_PATHS),
        Some("GridBox") => semantic_group("Grid Slot", &child_mount.slot, GRID_SLOT_PATHS),
        Some("FlowBox") => semantic_group("Flow Slot", &child_mount.slot, FLOW_SLOT_PATHS),
        _ => UiAssetInspectorSemanticGroup::default(),
    }
}

pub(crate) fn build_layout_semantic_group(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> UiAssetInspectorSemanticGroup {
    let Some(node) = selected_node(document, selection) else {
        return UiAssetInspectorSemanticGroup::default();
    };
    let Some(layout) = node.layout.as_ref() else {
        return UiAssetInspectorSemanticGroup::default();
    };
    match semantic_node_kind(node).as_deref() {
        Some("HorizontalBox") | Some("VerticalBox") => {
            semantic_group("Linear Layout", layout, LINEAR_LAYOUT_PATHS)
        }
        Some("ScrollableBox") => {
            semantic_group("Scrollable Layout", layout, SCROLLABLE_LAYOUT_PATHS)
        }
        _ => UiAssetInspectorSemanticGroup::default(),
    }
}

pub(crate) fn build_structured_slot_semantic_fields(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> UiAssetStructuredSlotSemanticFields {
    let Some(child_mount) = selected_child_mount(document, selection) else {
        return UiAssetStructuredSlotSemanticFields::default();
    };
    let Some(parent) = selected_parent_node(document, selection) else {
        return UiAssetStructuredSlotSemanticFields::default();
    };

    match semantic_parent_kind(parent).as_deref() {
        Some("HorizontalBox") => UiAssetStructuredSlotSemanticFields {
            kind: "HorizontalBox".to_string(),
            linear_main_weight: value_map_display_literal(&child_mount.slot, "layout.width.weight")
                .unwrap_or_default(),
            linear_main_stretch: value_map_string_literal(
                &child_mount.slot,
                "layout.width.stretch",
            )
            .unwrap_or_default(),
            linear_cross_weight: value_map_display_literal(
                &child_mount.slot,
                "layout.height.weight",
            )
            .unwrap_or_default(),
            linear_cross_stretch: value_map_string_literal(
                &child_mount.slot,
                "layout.height.stretch",
            )
            .unwrap_or_default(),
            ..UiAssetStructuredSlotSemanticFields::default()
        },
        Some("VerticalBox") => UiAssetStructuredSlotSemanticFields {
            kind: "VerticalBox".to_string(),
            linear_main_weight: value_map_display_literal(
                &child_mount.slot,
                "layout.height.weight",
            )
            .unwrap_or_default(),
            linear_main_stretch: value_map_string_literal(
                &child_mount.slot,
                "layout.height.stretch",
            )
            .unwrap_or_default(),
            linear_cross_weight: value_map_display_literal(
                &child_mount.slot,
                "layout.width.weight",
            )
            .unwrap_or_default(),
            linear_cross_stretch: value_map_string_literal(
                &child_mount.slot,
                "layout.width.stretch",
            )
            .unwrap_or_default(),
            ..UiAssetStructuredSlotSemanticFields::default()
        },
        Some("Overlay") => UiAssetStructuredSlotSemanticFields {
            kind: "Overlay".to_string(),
            overlay_anchor_x: value_map_display_literal(&child_mount.slot, "layout.anchor.x")
                .unwrap_or_default(),
            overlay_anchor_y: value_map_display_literal(&child_mount.slot, "layout.anchor.y")
                .unwrap_or_default(),
            overlay_pivot_x: value_map_display_literal(&child_mount.slot, "layout.pivot.x")
                .unwrap_or_default(),
            overlay_pivot_y: value_map_display_literal(&child_mount.slot, "layout.pivot.y")
                .unwrap_or_default(),
            overlay_position_x: value_map_display_literal(&child_mount.slot, "layout.position.x")
                .unwrap_or_default(),
            overlay_position_y: value_map_display_literal(&child_mount.slot, "layout.position.y")
                .unwrap_or_default(),
            overlay_z_index: value_map_display_literal(&child_mount.slot, "layout.z_index")
                .unwrap_or_default(),
            ..UiAssetStructuredSlotSemanticFields::default()
        },
        Some("GridBox") => UiAssetStructuredSlotSemanticFields {
            kind: "Grid".to_string(),
            grid_row: value_map_display_literal(&child_mount.slot, "row").unwrap_or_default(),
            grid_column: value_map_display_literal(&child_mount.slot, "column").unwrap_or_default(),
            grid_row_span: value_map_display_literal(&child_mount.slot, "row_span")
                .unwrap_or_default(),
            grid_column_span: value_map_display_literal(&child_mount.slot, "column_span")
                .unwrap_or_default(),
            ..UiAssetStructuredSlotSemanticFields::default()
        },
        Some("FlowBox") => UiAssetStructuredSlotSemanticFields {
            kind: "Flow".to_string(),
            flow_break_before: value_map_display_literal(&child_mount.slot, "break_before")
                .unwrap_or_default(),
            flow_alignment: value_map_display_literal(&child_mount.slot, "alignment")
                .unwrap_or_default(),
            ..UiAssetStructuredSlotSemanticFields::default()
        },
        _ => UiAssetStructuredSlotSemanticFields::default(),
    }
}

pub(crate) fn build_structured_layout_semantic_fields(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> UiAssetStructuredLayoutSemanticFields {
    let Some(node) = selected_node(document, selection) else {
        return UiAssetStructuredLayoutSemanticFields::default();
    };
    let Some(layout) = node.layout.as_ref() else {
        return UiAssetStructuredLayoutSemanticFields::default();
    };

    match semantic_node_kind(node).as_deref() {
        Some("HorizontalBox") => UiAssetStructuredLayoutSemanticFields {
            kind: "HorizontalBox".to_string(),
            box_gap: value_map_display_literal(layout, "container.gap").unwrap_or_default(),
            ..UiAssetStructuredLayoutSemanticFields::default()
        },
        Some("VerticalBox") => UiAssetStructuredLayoutSemanticFields {
            kind: "VerticalBox".to_string(),
            box_gap: value_map_display_literal(layout, "container.gap").unwrap_or_default(),
            ..UiAssetStructuredLayoutSemanticFields::default()
        },
        Some("ScrollableBox") => UiAssetStructuredLayoutSemanticFields {
            kind: "ScrollableBox".to_string(),
            box_gap: String::new(),
            scroll_axis: value_map_display_literal(layout, "container.axis").unwrap_or_default(),
            scroll_gap: value_map_display_literal(layout, "container.gap").unwrap_or_default(),
            scrollbar_visibility: value_map_display_literal(
                layout,
                "container.scrollbar_visibility",
            )
            .unwrap_or_default(),
            virtualization_item_extent: value_map_display_literal(
                layout,
                "container.virtualization.item_extent",
            )
            .unwrap_or_default(),
            virtualization_overscan: value_map_display_literal(
                layout,
                "container.virtualization.overscan",
            )
            .unwrap_or_default(),
            clip: value_map_display_literal(layout, "clip").unwrap_or_default(),
        },
        _ => UiAssetStructuredLayoutSemanticFields::default(),
    }
}

pub(crate) fn reconcile_selected_semantic_path(
    entries: &[UiAssetInspectorSemanticEntry],
    current: Option<&str>,
) -> Option<String> {
    current
        .filter(|path| entries.iter().any(|entry| entry.path == *path))
        .map(str::to_string)
}

pub(crate) fn set_selected_slot_semantic_value(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    path: &str,
    literal: &str,
) -> bool {
    let Some(child_mount) = selected_child_mount_mut(document, selection) else {
        return false;
    };
    set_value_in_map(&mut child_mount.slot, path, literal)
}

pub(crate) fn delete_selected_slot_semantic(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    path: &str,
) -> bool {
    let Some(child_mount) = selected_child_mount_mut(document, selection) else {
        return false;
    };
    remove_value_in_map(&mut child_mount.slot, path)
}

pub(crate) fn set_selected_layout_semantic_value(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    path: &str,
    literal: &str,
) -> bool {
    let Some(node) = selected_node_mut(document, selection) else {
        return false;
    };
    let current = node
        .layout
        .as_ref()
        .and_then(|layout| value_map_literal(layout, path));
    let next = parse_declaration_literal(literal);
    let next_literal = next.as_ref().map(Value::to_string);
    if current == next_literal {
        return false;
    }

    match next {
        Some(value) => {
            let layout = node.layout.get_or_insert_with(Default::default);
            set_path_value(layout, &split_path(path), value);
            true
        }
        None => {
            let Some(layout) = node.layout.as_mut() else {
                return false;
            };
            let removed = remove_path_value(layout, &split_path(path));
            if removed && layout.is_empty() {
                node.layout = None;
            }
            removed
        }
    }
}

pub(crate) fn delete_selected_layout_semantic(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    path: &str,
) -> bool {
    let Some(node) = selected_node_mut(document, selection) else {
        return false;
    };
    let Some(layout) = node.layout.as_mut() else {
        return false;
    };
    let removed = remove_path_value(layout, &split_path(path));
    if removed && layout.is_empty() {
        node.layout = None;
    }
    removed
}

const OVERLAY_SLOT_PATHS: &[&str] = &[
    "layout.anchor.x",
    "layout.anchor.y",
    "layout.pivot.x",
    "layout.pivot.y",
    "layout.position.x",
    "layout.position.y",
    "layout.z_index",
];

const GRID_SLOT_PATHS: &[&str] = &["row", "column", "row_span", "column_span"];

const FLOW_SLOT_PATHS: &[&str] = &["break_before", "alignment"];

const LINEAR_SLOT_PATHS: &[&str] = &[
    "layout.width.weight",
    "layout.width.stretch",
    "layout.height.weight",
    "layout.height.stretch",
];

const LINEAR_LAYOUT_PATHS: &[&str] = &["container.gap"];

const SCROLLABLE_LAYOUT_PATHS: &[&str] = &[
    "container.axis",
    "container.gap",
    "container.scrollbar_visibility",
    "container.virtualization.item_extent",
    "container.virtualization.overscan",
    "clip",
];

fn semantic_group(
    title: &str,
    values: &BTreeMap<String, Value>,
    paths: &[&str],
) -> UiAssetInspectorSemanticGroup {
    let entries = paths
        .iter()
        .filter_map(|path| {
            value_map_value(values, path).map(|value| UiAssetInspectorSemanticEntry {
                path: (*path).to_string(),
                literal: display_literal(value),
            })
        })
        .collect::<Vec<_>>();
    if entries.is_empty() {
        UiAssetInspectorSemanticGroup::default()
    } else {
        UiAssetInspectorSemanticGroup {
            title: title.to_string(),
            entries,
        }
    }
}

fn display_literal(value: &Value) -> String {
    match value {
        Value::Float(number) if number.fract() == 0.0 => format!("{number:.0}"),
        _ => value.to_string(),
    }
}

fn semantic_parent_kind(node: &UiNodeDefinition) -> Option<String> {
    semantic_node_kind(node)
}

fn semantic_node_kind(node: &UiNodeDefinition) -> Option<String> {
    node.layout
        .as_ref()
        .and_then(|layout| value_map_value(layout, "container.kind"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| node.widget_type.clone())
}

fn set_value_in_map(values: &mut BTreeMap<String, Value>, path: &str, literal: &str) -> bool {
    let current = value_map_literal(values, path);
    let next = parse_declaration_literal(literal);
    let next_literal = next.as_ref().map(Value::to_string);
    if current == next_literal {
        return false;
    }

    match next {
        Some(value) => {
            set_path_value(values, &split_path(path), value);
            true
        }
        None => remove_path_value(values, &split_path(path)),
    }
}

fn remove_value_in_map(values: &mut BTreeMap<String, Value>, path: &str) -> bool {
    remove_path_value(values, &split_path(path))
}

fn split_path(path: &str) -> Vec<String> {
    path.split('.')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect()
}

fn value_map_literal(values: &BTreeMap<String, Value>, path: &str) -> Option<String> {
    value_map_value(values, path).map(Value::to_string)
}

fn value_map_display_literal(values: &BTreeMap<String, Value>, path: &str) -> Option<String> {
    value_map_value(values, path).map(display_literal)
}

fn value_map_string_literal(values: &BTreeMap<String, Value>, path: &str) -> Option<String> {
    value_map_value(values, path).map(|value| match value {
        Value::String(value) => value.clone(),
        other => display_literal(other),
    })
}

fn value_map_value<'a>(values: &'a BTreeMap<String, Value>, path: &str) -> Option<&'a Value> {
    let segments = split_path(path);
    let (first, rest) = segments.split_first()?;
    let value = values.get(first)?;
    if rest.is_empty() {
        return Some(value);
    }
    let Value::Table(table) = value else {
        return None;
    };
    table_value_at_path(table, rest)
}

fn table_value_at_path<'a>(table: &'a Map<String, Value>, path: &[String]) -> Option<&'a Value> {
    let (first, rest) = path.split_first()?;
    let value = table.get(first)?;
    if rest.is_empty() {
        return Some(value);
    }
    let Value::Table(child) = value else {
        return None;
    };
    table_value_at_path(child, rest)
}

fn set_path_value(values: &mut BTreeMap<String, Value>, path: &[String], value: Value) {
    let Some((first, rest)) = path.split_first() else {
        return;
    };
    if rest.is_empty() {
        let _ = values.insert(first.clone(), value);
        return;
    }

    let entry = values
        .entry(first.clone())
        .or_insert_with(|| Value::Table(Map::new()));
    if !matches!(entry, Value::Table(_)) {
        *entry = Value::Table(Map::new());
    }
    let Value::Table(table) = entry else {
        unreachable!("entry should be a table before recursion");
    };
    set_table_path_value(table, rest, value);
}

fn set_table_path_value(values: &mut Map<String, Value>, path: &[String], value: Value) {
    let Some((first, rest)) = path.split_first() else {
        return;
    };
    if rest.is_empty() {
        let _ = values.insert(first.clone(), value);
        return;
    }

    let entry = values
        .entry(first.clone())
        .or_insert_with(|| Value::Table(Map::new()));
    if !matches!(entry, Value::Table(_)) {
        *entry = Value::Table(Map::new());
    }
    let Value::Table(table) = entry else {
        unreachable!("entry should be a table before recursion");
    };
    set_table_path_value(table, rest, value);
}

fn remove_path_value(values: &mut BTreeMap<String, Value>, path: &[String]) -> bool {
    let Some((first, rest)) = path.split_first() else {
        return false;
    };
    if rest.is_empty() {
        return values.remove(first).is_some();
    }

    let Some(entry) = values.get_mut(first) else {
        return false;
    };
    let Value::Table(table) = entry else {
        return false;
    };
    let removed = remove_table_path_value(table, rest);
    if removed && table.is_empty() {
        let _ = values.remove(first);
    }
    removed
}

fn remove_table_path_value(values: &mut Map<String, Value>, path: &[String]) -> bool {
    let Some((first, rest)) = path.split_first() else {
        return false;
    };
    if rest.is_empty() {
        return values.remove(first).is_some();
    }

    let Some(entry) = values.get_mut(first) else {
        return false;
    };
    let Value::Table(table) = entry else {
        return false;
    };
    let removed = remove_table_path_value(table, rest);
    if removed && table.is_empty() {
        let _ = values.remove(first);
    }
    removed
}

fn selected_node<'a>(
    document: &'a UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<&'a UiNodeDefinition> {
    selection
        .primary_node_id
        .as_deref()
        .and_then(|node_id| document.nodes.get(node_id))
}

fn selected_node_mut<'a>(
    document: &'a mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<&'a mut UiNodeDefinition> {
    selection
        .primary_node_id
        .as_deref()
        .and_then(|node_id| document.nodes.get_mut(node_id))
}

fn selected_parent_node<'a>(
    document: &'a UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<&'a UiNodeDefinition> {
    selection
        .parent_node_id
        .as_deref()
        .and_then(|node_id| document.nodes.get(node_id))
}

fn selected_child_mount<'a>(
    document: &'a UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<&'a UiChildMount> {
    let node_id = selection.primary_node_id.as_deref()?;
    document.nodes.values().find_map(|node| {
        node.children
            .iter()
            .find(|child_mount| child_mount.child == node_id)
    })
}

fn selected_child_mount_mut<'a>(
    document: &'a mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<&'a mut UiChildMount> {
    let node_id = selection.primary_node_id.as_deref()?;
    document.nodes.values_mut().find_map(|node| {
        node.children
            .iter_mut()
            .find(|child_mount| child_mount.child == node_id)
    })
}
