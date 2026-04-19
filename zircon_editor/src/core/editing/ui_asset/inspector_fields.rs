use crate::ui::UiDesignerSelectionModel;
use toml::{map::Map, Value};
use zircon_runtime::ui::template::UiAssetDocument;
use zircon_runtime::ui::template::UiChildMount;
use zircon_runtime::ui::template::{UiNodeDefinition, UiNodeDefinitionKind};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiAssetInspectorFields {
    pub selected_node_id: String,
    pub parent_node_id: String,
    pub mount: String,
    pub slot_padding: String,
    pub slot_width_preferred: String,
    pub slot_height_preferred: String,
    pub layout_width_preferred: String,
    pub layout_height_preferred: String,
    pub widget_kind: String,
    pub widget_label: String,
    pub control_id: String,
    pub text_prop: String,
    pub can_edit_control_id: bool,
    pub can_edit_text_prop: bool,
}

pub(crate) fn build_inspector_fields(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> UiAssetInspectorFields {
    let Some(node_id) = selection.primary_node_id.as_deref() else {
        return UiAssetInspectorFields::default();
    };
    let Some(node) = document.nodes.get(node_id) else {
        return UiAssetInspectorFields::default();
    };
    let slot_mount = selected_child_mount(document, selection);
    let editable = !matches!(node.kind, UiNodeDefinitionKind::Slot);
    UiAssetInspectorFields {
        selected_node_id: node_id.to_string(),
        parent_node_id: selection.parent_node_id.clone().unwrap_or_default(),
        mount: slot_mount
            .and_then(|child_mount| child_mount.mount.clone())
            .or_else(|| selection.mount.clone())
            .unwrap_or_default(),
        slot_padding: slot_mount
            .and_then(|child_mount| value_map_literal(&child_mount.slot, &["padding"]))
            .unwrap_or_default(),
        slot_width_preferred: slot_mount
            .and_then(|child_mount| {
                value_map_literal(&child_mount.slot, &["layout", "width", "preferred"])
            })
            .unwrap_or_default(),
        slot_height_preferred: slot_mount
            .and_then(|child_mount| {
                value_map_literal(&child_mount.slot, &["layout", "height", "preferred"])
            })
            .unwrap_or_default(),
        layout_width_preferred: node
            .layout
            .as_ref()
            .and_then(|layout| value_map_literal(layout, &["width", "preferred"]))
            .unwrap_or_default(),
        layout_height_preferred: node
            .layout
            .as_ref()
            .and_then(|layout| value_map_literal(layout, &["height", "preferred"]))
            .unwrap_or_default(),
        widget_kind: node_kind_label(node.kind).to_string(),
        widget_label: node_label(node),
        control_id: node.control_id.clone().unwrap_or_default(),
        text_prop: text_property_value(node),
        can_edit_control_id: editable,
        can_edit_text_prop: editable,
    }
}

pub(crate) fn set_selected_node_control_id(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    control_id: &str,
) -> bool {
    let Some(node) = selected_node_mut(document, selection) else {
        return false;
    };
    if matches!(node.kind, UiNodeDefinitionKind::Slot) {
        return false;
    }
    let next = normalized_control_id(control_id);
    if node.control_id == next {
        return false;
    }
    node.control_id = next;
    true
}

pub(crate) fn set_selected_node_mount(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    mount: &str,
) -> bool {
    let Some(child_mount) = selected_child_mount_mut(document, selection) else {
        return false;
    };
    let next = normalized_mount(mount);
    if child_mount.mount == next {
        return false;
    }
    child_mount.mount = next;
    true
}

pub(crate) fn set_selected_node_text_property(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    text: &str,
) -> bool {
    let Some(node) = selected_node_mut(document, selection) else {
        return false;
    };
    if matches!(node.kind, UiNodeDefinitionKind::Slot) {
        return false;
    }
    if text.is_empty() {
        return node.props.remove("text").is_some();
    }

    let next = Value::String(text.to_string());
    if node.props.get("text") == Some(&next) {
        return false;
    }
    let _ = node.props.insert("text".to_string(), next);
    true
}

pub(crate) fn set_selected_node_slot_padding(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    literal: &str,
) -> Result<bool, &'static str> {
    set_selected_child_numeric_slot_value(document, selection, &["padding"], literal)
        .map_err(|_| "slot.padding")
}

pub(crate) fn set_selected_node_slot_width_preferred(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    literal: &str,
) -> Result<bool, &'static str> {
    set_selected_child_numeric_slot_value(
        document,
        selection,
        &["layout", "width", "preferred"],
        literal,
    )
    .map_err(|_| "slot.layout.width.preferred")
}

pub(crate) fn set_selected_node_slot_height_preferred(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    literal: &str,
) -> Result<bool, &'static str> {
    set_selected_child_numeric_slot_value(
        document,
        selection,
        &["layout", "height", "preferred"],
        literal,
    )
    .map_err(|_| "slot.layout.height.preferred")
}

pub(crate) fn set_selected_node_layout_width_preferred(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    literal: &str,
) -> Result<bool, &'static str> {
    set_selected_node_numeric_layout_value(document, selection, &["width", "preferred"], literal)
        .map_err(|_| "layout.width.preferred")
}

pub(crate) fn set_selected_node_layout_height_preferred(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    literal: &str,
) -> Result<bool, &'static str> {
    set_selected_node_numeric_layout_value(document, selection, &["height", "preferred"], literal)
        .map_err(|_| "layout.height.preferred")
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

fn normalized_control_id(control_id: &str) -> Option<String> {
    let trimmed = control_id.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn normalized_mount(mount: &str) -> Option<String> {
    let trimmed = mount.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn text_property_value(node: &UiNodeDefinition) -> String {
    node.props
        .get("text")
        .map(|value| match value {
            Value::String(text) => text.clone(),
            other => other.to_string(),
        })
        .unwrap_or_default()
}

fn node_kind_label(kind: UiNodeDefinitionKind) -> &'static str {
    match kind {
        UiNodeDefinitionKind::Native => "Native",
        UiNodeDefinitionKind::Component => "Component",
        UiNodeDefinitionKind::Reference => "Reference",
        UiNodeDefinitionKind::Slot => "Slot",
    }
}

fn node_label(node: &UiNodeDefinition) -> String {
    node.widget_type
        .clone()
        .or_else(|| node.component.clone())
        .or_else(|| {
            node.component_ref.as_ref().map(|reference| {
                reference
                    .split_once('#')
                    .map(|(_, component)| component.to_string())
                    .unwrap_or_else(|| reference.clone())
            })
        })
        .or_else(|| node.slot_name.clone())
        .unwrap_or_else(|| "Node".to_string())
}

fn set_selected_child_numeric_slot_value(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    path: &[&str],
    literal: &str,
) -> Result<bool, ()> {
    let Some(child_mount) = selected_child_mount_mut(document, selection) else {
        return Ok(false);
    };
    let current = value_map_literal(&child_mount.slot, path);
    let next = parse_optional_numeric_literal(literal)?;
    let next_literal = next.as_ref().map(Value::to_string);
    if current == next_literal {
        return Ok(false);
    }

    if let Some(value) = next {
        set_value_map_path(&mut child_mount.slot, path, value);
    } else if !remove_value_map_path(&mut child_mount.slot, path) {
        return Ok(false);
    }
    Ok(true)
}

fn set_selected_node_numeric_layout_value(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    path: &[&str],
    literal: &str,
) -> Result<bool, ()> {
    let Some(node) = selected_node_mut(document, selection) else {
        return Ok(false);
    };
    let current = node
        .layout
        .as_ref()
        .and_then(|layout| value_map_literal(layout, path));
    let next = parse_optional_numeric_literal(literal)?;
    let next_literal = next.as_ref().map(Value::to_string);
    if current == next_literal {
        return Ok(false);
    }

    if let Some(value) = next {
        let layout = node.layout.get_or_insert_with(Default::default);
        set_value_map_path(layout, path, value);
        return Ok(true);
    }

    let Some(layout) = node.layout.as_mut() else {
        return Ok(false);
    };
    if !remove_value_map_path(layout, path) {
        return Ok(false);
    }
    if layout.is_empty() {
        node.layout = None;
    }
    Ok(true)
}

fn value_map_literal(
    values: &std::collections::BTreeMap<String, Value>,
    path: &[&str],
) -> Option<String> {
    value_map_value(values, path).map(Value::to_string)
}

fn value_map_value<'a>(
    values: &'a std::collections::BTreeMap<String, Value>,
    path: &[&str],
) -> Option<&'a Value> {
    let (first, rest) = path.split_first()?;
    let value = values.get(*first)?;
    if rest.is_empty() {
        return Some(value);
    }
    let Value::Table(table) = value else {
        return None;
    };
    table_value_at_path(table, rest)
}

fn table_value_at_path<'a>(table: &'a Map<String, Value>, path: &[&str]) -> Option<&'a Value> {
    let (first, rest) = path.split_first()?;
    let value = table.get(*first)?;
    if rest.is_empty() {
        return Some(value);
    }
    let Value::Table(child) = value else {
        return None;
    };
    table_value_at_path(child, rest)
}

fn parse_optional_numeric_literal(literal: &str) -> Result<Option<Value>, ()> {
    let trimmed = literal.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let parsed = toml::from_str::<toml::Table>(&format!("value = {trimmed}"))
        .ok()
        .and_then(|table| table.get("value").cloned())
        .ok_or(())?;
    match parsed {
        Value::Integer(_) | Value::Float(_) => Ok(Some(parsed)),
        _ => Err(()),
    }
}

fn set_value_map_path(
    values: &mut std::collections::BTreeMap<String, Value>,
    path: &[&str],
    value: Value,
) {
    let Some((first, rest)) = path.split_first() else {
        return;
    };
    if rest.is_empty() {
        let _ = values.insert((*first).to_string(), value);
        return;
    }

    let entry = values
        .entry((*first).to_string())
        .or_insert_with(|| Value::Table(Map::new()));
    if !matches!(entry, Value::Table(_)) {
        *entry = Value::Table(Map::new());
    }
    let Value::Table(table) = entry else {
        unreachable!("slot path entry should be table");
    };
    set_table_path(table, rest, value);
}

fn set_table_path(table: &mut Map<String, Value>, path: &[&str], value: Value) {
    let Some((first, rest)) = path.split_first() else {
        return;
    };
    if rest.is_empty() {
        let _ = table.insert((*first).to_string(), value);
        return;
    }

    let entry = table
        .entry((*first).to_string())
        .or_insert_with(|| Value::Table(Map::new()));
    if !matches!(entry, Value::Table(_)) {
        *entry = Value::Table(Map::new());
    }
    let Value::Table(child) = entry else {
        unreachable!("slot path entry should be table");
    };
    set_table_path(child, rest, value);
}

fn remove_value_map_path(
    values: &mut std::collections::BTreeMap<String, Value>,
    path: &[&str],
) -> bool {
    let Some((first, rest)) = path.split_first() else {
        return false;
    };
    if rest.is_empty() {
        return values.remove(*first).is_some();
    }

    let Some(value) = values.get_mut(*first) else {
        return false;
    };
    let Value::Table(table) = value else {
        return false;
    };
    let removed = remove_table_path(table, rest);
    if removed && table.is_empty() {
        let _ = values.remove(*first);
    }
    removed
}

fn remove_table_path(table: &mut Map<String, Value>, path: &[&str]) -> bool {
    let Some((first, rest)) = path.split_first() else {
        return false;
    };
    if rest.is_empty() {
        return table.remove(*first).is_some();
    }

    let Some(value) = table.get_mut(*first) else {
        return false;
    };
    let Value::Table(child) = value else {
        return false;
    };
    let removed = remove_table_path(child, rest);
    if removed && child.is_empty() {
        let _ = table.remove(*first);
    }
    removed
}
