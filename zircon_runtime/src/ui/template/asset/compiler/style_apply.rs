use std::collections::BTreeMap;

use toml::Value;

use zircon_runtime_interface::ui::template::{
    UiAssetError, UiSelector, UiStyleDeclarationBlock, UiTemplateNode,
};

use super::super::style::{UiRuntimeSelectorMatchExt, UiSelectorMatchNode};
use super::ui_document_compiler::ResolvedStyleSheet;
use super::value_normalizer::{merge_value_maps, merge_value_maps_resolved};

mod mui_collection_classes;
mod mui_display_surface_classes;
mod mui_form_classes;
mod mui_lab_classes;
mod mui_layout_classes;
mod mui_navigation_classes;
mod mui_selection_classes;
mod mui_surface_child_classes;
mod mui_x_classes;

#[derive(Clone)]
pub(super) struct ParsedStyleRule {
    selector: UiSelector,
    specificity: usize,
    order: usize,
    set: UiStyleDeclarationBlock,
    tokens: BTreeMap<String, Value>,
}

pub(super) fn build_style_plan(
    sheets: &[ResolvedStyleSheet],
) -> Result<Vec<ParsedStyleRule>, UiAssetError> {
    let mut rules = Vec::new();
    let mut order = 0;
    for sheet in sheets {
        for rule in &sheet.stylesheet.rules {
            let selector = UiSelector::parse(&rule.selector)?;
            rules.push(ParsedStyleRule {
                specificity: selector.specificity(),
                selector,
                order,
                set: rule.set.clone(),
                tokens: sheet.tokens.clone(),
            });
            order += 1;
        }
    }
    Ok(rules)
}

pub(super) fn apply_styles_to_tree(
    node: &mut UiTemplateNode,
    rules: &[ParsedStyleRule],
    path: &mut Vec<StylePathEntry>,
) {
    apply_mui_root_slot_props_to_node(node);
    append_mui_style_classes(node);
    path.push(StylePathEntry::from_node(node, path.is_empty()));

    let path_snapshot: Vec<_> = path.iter().map(StylePathEntry::as_match_node).collect();
    let mut matched: Vec<_> = rules
        .iter()
        .filter(|rule| rule.selector.matches_path(&path_snapshot))
        .cloned()
        .collect();
    matched.sort_by_key(|rule| (rule.specificity, rule.order));
    for rule in matched {
        merge_value_maps_resolved(
            &mut node.attributes,
            &rule.set.self_values,
            &rule.tokens,
            &BTreeMap::new(),
        );
        merge_value_maps_resolved(
            &mut node.slot_attributes,
            &rule.set.slot,
            &rule.tokens,
            &BTreeMap::new(),
        );
    }

    apply_mui_sx_to_node(node);
    if !node.style_overrides.is_empty() {
        let inline = node.style_overrides.clone();
        merge_value_maps(&mut node.attributes, &inline);
    }

    apply_mui_child_slot_props(node);
    for child in &mut node.children {
        apply_styles_to_tree(child, rules, path);
    }

    let _ = path.pop();
}

#[derive(Clone)]
pub(super) struct StylePathEntry {
    component: String,
    control_id: Option<String>,
    classes: Vec<String>,
    is_host: bool,
    states: Vec<String>,
}

impl StylePathEntry {
    fn from_node(node: &UiTemplateNode, is_host: bool) -> Self {
        Self {
            component: node.component.clone().unwrap_or_default(),
            control_id: node.control_id.clone(),
            classes: node.classes.clone(),
            is_host,
            states: selector_states(node),
        }
    }

    fn as_match_node(&self) -> UiSelectorMatchNode<'_> {
        UiSelectorMatchNode {
            component: &self.component,
            control_id: self.control_id.as_deref(),
            classes: &self.classes,
            is_host: self.is_host,
            states: &self.states,
        }
    }
}

pub(super) fn apply_mui_sx_to_node(node: &mut UiTemplateNode) {
    let sx = map_attribute_any(node, &["mui_sx", "sx"]);
    if sx.is_empty() {
        return;
    };

    merge_value_maps(&mut node.attributes, &sx);
    let mut overrides = sx;
    merge_value_maps(&mut overrides, &node.style_overrides);
    node.style_overrides = overrides;
}

pub(super) fn apply_mui_root_slot_props_to_node(node: &mut UiTemplateNode) {
    let root_props = nested_map_attribute_any(node, &["mui_slot_props", "slotProps"], "root");
    if root_props.is_empty() {
        return;
    };
    merge_value_maps(&mut node.attributes, &root_props);
}

pub(super) fn apply_mui_child_slot_props(node: &mut UiTemplateNode) {
    let slot_props = map_attribute_any(node, &["mui_slot_props", "slotProps"]);
    let slot_components = map_attribute_any(node, &["mui_slots", "slots"]);
    let slot_classes = map_attribute(node, "classes");
    let owner_prefix = node
        .component
        .as_deref()
        .filter(|component| !component.is_empty())
        .map(|component| format!("Mui{component}"));
    let owner_component = node.component.clone().unwrap_or_default();
    let owner_attributes = node.attributes.clone();
    if slot_props.is_empty()
        && slot_components.is_empty()
        && slot_classes.is_empty()
        && owner_prefix.is_none()
    {
        return;
    }

    for child in &mut node.children {
        if owner_component == "Skeleton" {
            mui_display_surface_classes::append_skeleton_child_metadata(child);
        }
        let Some(slot_name) = mui_slot_name(child) else {
            continue;
        };
        apply_mui_slot_contract_to_child(
            child,
            &slot_name,
            &owner_component,
            &owner_attributes,
            owner_prefix.as_deref(),
            &slot_props,
            &slot_components,
            &slot_classes,
        );
    }

    for (slot_name, children) in &mut node.slots {
        for child in children {
            if owner_component == "Skeleton" {
                mui_display_surface_classes::append_skeleton_child_metadata(child);
            }
            child
                .slot_attributes
                .entry("mui_slot".to_string())
                .or_insert_with(|| Value::String(slot_name.clone()));
            apply_mui_slot_contract_to_child(
                child,
                slot_name,
                &owner_component,
                &owner_attributes,
                owner_prefix.as_deref(),
                &slot_props,
                &slot_components,
                &slot_classes,
            );
        }
    }
}

pub(super) fn append_mui_style_classes(node: &mut UiTemplateNode) {
    let component = node.component.clone().unwrap_or_default();
    if component.is_empty() {
        return;
    }
    let prefix = format!("Mui{component}");
    append_class(&mut node.classes, format!("{prefix}-root"));

    for class_name in string_array_attribute(node, "mui_classes") {
        append_class(&mut node.classes, class_name);
    }
    for class_name in string_list_attribute(node, "className") {
        append_class(&mut node.classes, class_name);
    }
    for class_name in classes_map_slot_classes(node, "root") {
        append_class(&mut node.classes, class_name);
    }

    if should_emit_generic_variant_class(&component) {
        if let Some(variant) = string_attribute_any_prefer_non_default(
            node,
            &[
                ("mui_variant", ""),
                ("button_variant", "default"),
                ("variant", "default"),
            ],
        ) {
            append_class(&mut node.classes, format!("{prefix}-{variant}"));
        }
    }
    if should_emit_generic_color_class(&component) {
        if let Some(color) = string_attribute_any_prefer_non_default(
            node,
            &[
                ("mui_color", "primary"),
                ("button_color", "primary"),
                ("color", "primary"),
            ],
        ) {
            append_class(
                &mut node.classes,
                format!("{prefix}-color{}", pascal_case(&color)),
            );
        }
    }
    if should_emit_generic_size_class(&component) {
        if let Some(size) = string_attribute_any_prefer_non_default(
            node,
            &[
                ("mui_size", "medium"),
                ("button_size", "medium"),
                ("size", "medium"),
            ],
        ) {
            append_class(
                &mut node.classes,
                format!("{prefix}-size{}", pascal_case(&size)),
            );
        }
    }

    append_mui_component_specific_classes(node, &component, &prefix);

    for class_name in mui_state_classes(node) {
        append_class(&mut node.classes, class_name);
    }
}

fn append_mui_component_specific_classes(node: &mut UiTemplateNode, component: &str, prefix: &str) {
    if mui_layout_classes::append_layout_component_classes(node, component, prefix) {
        return;
    }
    if mui_form_classes::append_form_component_classes(node, component, prefix) {
        return;
    }
    if mui_selection_classes::append_component_classes(node, component, prefix) {
        return;
    }
    if mui_collection_classes::append_component_classes(node, component, prefix) {
        return;
    }
    if mui_lab_classes::append_component_classes(node, component, prefix) {
        return;
    }
    if mui_x_classes::append_component_classes(node, component, prefix) {
        return;
    }
    if mui_surface_child_classes::append_component_classes(node, component, prefix) {
        return;
    }
    if mui_display_surface_classes::append_component_classes(node, component, prefix) {
        return;
    }

    match component {
        "BottomNavigationAction" => {
            mui_navigation_classes::append_bottom_navigation_action_classes(node, prefix)
        }
        "Link" => mui_navigation_classes::append_link_classes(node, prefix),
        "MenuItem" => mui_navigation_classes::append_menu_item_classes(node, prefix),
        "Pagination" => mui_navigation_classes::append_pagination_classes(node, prefix),
        "PaginationItem" => mui_navigation_classes::append_pagination_item_classes(node, prefix),
        "Step" => mui_navigation_classes::append_step_classes(node, prefix),
        "StepButton" => mui_navigation_classes::append_step_button_classes(node, prefix),
        "StepConnector" => mui_navigation_classes::append_step_connector_classes(node, prefix),
        "StepContent" => mui_navigation_classes::append_step_content_classes(node, prefix),
        "StepLabel" => mui_navigation_classes::append_step_label_classes(node, prefix),
        "Stepper" => mui_navigation_classes::append_stepper_classes(node, prefix),
        "Tab" => mui_navigation_classes::append_tab_classes(node, prefix),
        "Tabs" => mui_navigation_classes::append_tabs_classes(node, prefix),
        "TransferList" => mui_navigation_classes::append_transfer_list_classes(node, prefix),
        _ => {}
    }
}

fn should_emit_generic_variant_class(component: &str) -> bool {
    if mui_layout_classes::suppresses_generic_classes(component)
        || mui_form_classes::suppresses_generic_classes(component)
        || mui_selection_classes::suppresses_generic_classes(component)
        || mui_collection_classes::suppresses_generic_classes(component)
        || mui_lab_classes::suppresses_generic_classes(component)
        || mui_x_classes::suppresses_generic_classes(component)
        || mui_surface_child_classes::suppresses_generic_classes(component)
    {
        return false;
    }

    !matches!(
        component,
        "Badge"
            | "BottomNavigation"
            | "BottomNavigationAction"
            | "Breadcrumbs"
            | "Card"
            | "Icon"
            | "Link"
            | "Menu"
            | "MenuItem"
            | "Pagination"
            | "PaginationItem"
            | "Step"
            | "StepButton"
            | "StepConnector"
            | "StepContent"
            | "StepIcon"
            | "StepLabel"
            | "Stepper"
            | "SvgIcon"
            | "Tab"
            | "Tabs"
            | "TransferList"
    )
}

fn should_emit_generic_color_class(component: &str) -> bool {
    if mui_layout_classes::suppresses_generic_classes(component)
        || mui_form_classes::suppresses_generic_classes(component)
        || mui_selection_classes::suppresses_generic_classes(component)
        || mui_collection_classes::suppresses_generic_classes(component)
        || mui_lab_classes::suppresses_generic_classes(component)
        || mui_x_classes::suppresses_generic_classes(component)
        || mui_surface_child_classes::suppresses_generic_classes(component)
    {
        return false;
    }

    !matches!(
        component,
        "Accordion"
            | "Alert"
            | "AlertTitle"
            | "AppBar"
            | "Avatar"
            | "Badge"
            | "BottomNavigation"
            | "BottomNavigationAction"
            | "Breadcrumbs"
            | "Card"
            | "CardActionArea"
            | "CardActions"
            | "CardContent"
            | "CardHeader"
            | "CardMedia"
            | "Divider"
            | "Icon"
            | "ImageList"
            | "Link"
            | "List"
            | "Menu"
            | "MenuItem"
            | "Paper"
            | "Pagination"
            | "PaginationItem"
            | "Skeleton"
            | "Snackbar"
            | "SnackbarContent"
            | "Step"
            | "StepButton"
            | "StepConnector"
            | "StepContent"
            | "StepIcon"
            | "StepLabel"
            | "Stepper"
            | "SvgIcon"
            | "Tab"
            | "Tabs"
            | "Table"
            | "Toolbar"
            | "TransferList"
            | "Typography"
    )
}

fn should_emit_generic_size_class(component: &str) -> bool {
    if mui_layout_classes::suppresses_generic_classes(component)
        || mui_form_classes::suppresses_generic_classes(component)
        || mui_selection_classes::suppresses_generic_classes(component)
        || mui_collection_classes::suppresses_generic_classes(component)
        || mui_lab_classes::suppresses_generic_classes(component)
        || mui_x_classes::suppresses_generic_classes(component)
        || mui_surface_child_classes::suppresses_generic_classes(component)
    {
        return false;
    }

    !matches!(
        component,
        "Accordion"
            | "Alert"
            | "AlertTitle"
            | "AppBar"
            | "Avatar"
            | "Badge"
            | "BottomNavigation"
            | "BottomNavigationAction"
            | "Breadcrumbs"
            | "Card"
            | "CardActionArea"
            | "CardActions"
            | "CardContent"
            | "CardHeader"
            | "CardMedia"
            | "Divider"
            | "Icon"
            | "ImageList"
            | "Link"
            | "List"
            | "Menu"
            | "MenuItem"
            | "Paper"
            | "Pagination"
            | "PaginationItem"
            | "Skeleton"
            | "Snackbar"
            | "SnackbarContent"
            | "Step"
            | "StepButton"
            | "StepConnector"
            | "StepContent"
            | "StepIcon"
            | "StepLabel"
            | "Stepper"
            | "SvgIcon"
            | "Tab"
            | "Tabs"
            | "Table"
            | "Toolbar"
            | "TransferList"
            | "Typography"
    )
}

fn apply_mui_slot_contract_to_child(
    child: &mut UiTemplateNode,
    slot_name: &str,
    owner_component: &str,
    owner_attributes: &BTreeMap<String, Value>,
    owner_prefix: Option<&str>,
    slot_props: &BTreeMap<String, Value>,
    slot_components: &BTreeMap<String, Value>,
    slot_classes: &BTreeMap<String, Value>,
) {
    if let Some(prefix) = owner_prefix {
        append_class(&mut child.classes, format!("{prefix}-{slot_name}"));
    }
    append_mui_owner_slot_utility_classes(child, owner_component, owner_attributes, slot_name);
    if let Some(component) = slot_components
        .get(slot_name)
        .and_then(Value::as_str)
        .filter(|component| !component.trim().is_empty())
    {
        let _ = child.attributes.insert(
            "mui_slot_component".to_string(),
            Value::String(component.trim().to_string()),
        );
    }
    if let Some(props) = slot_props.get(slot_name).and_then(value_as_map) {
        merge_value_maps(&mut child.attributes, &props);
    }
    for class_name in string_list_from_value(slot_classes.get(slot_name)) {
        append_class(&mut child.classes, class_name);
    }
}

fn append_mui_owner_slot_utility_classes(
    child: &mut UiTemplateNode,
    owner_component: &str,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) {
    if mui_layout_classes::append_layout_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_form_classes::append_form_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_selection_classes::append_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_collection_classes::append_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_x_classes::append_slot_classes(child, owner_component, owner_attributes, slot_name) {
        return;
    }
    if mui_surface_child_classes::append_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_display_surface_classes::append_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }

    match (owner_component, slot_name) {
        ("BottomNavigationAction", "label") => {
            mui_navigation_classes::append_bottom_navigation_action_label_slot_classes(
                child,
                owner_attributes,
            )
        }
        ("StepConnector", "line") => {
            mui_navigation_classes::append_step_connector_line_slot_classes(child, owner_attributes)
        }
        ("StepLabel", "label" | "iconContainer" | "labelContainer") => {
            mui_navigation_classes::append_step_label_slot_classes(
                child,
                owner_attributes,
                slot_name,
            )
        }
        ("Tabs", "list" | "scroller" | "scrollButtons") => {
            mui_navigation_classes::append_tabs_slot_classes(child, owner_attributes, slot_name)
        }
        ("TransferList", "source" | "target" | "actions") => {
            mui_navigation_classes::append_transfer_list_slot_classes(
                child,
                owner_attributes,
                slot_name,
            )
        }
        _ => {}
    }
}

fn selector_states(node: &UiTemplateNode) -> Vec<String> {
    let mut states = Vec::new();
    for (attribute, aliases) in [
        ("hovered", &["hover", "hovered"][..]),
        ("pressed", &["press", "pressed", "active"][..]),
        ("focused", &["focus", "focused"][..]),
        ("selected", &["selected"][..]),
        ("checked", &["checked"][..]),
        ("expanded", &["expanded"][..]),
        ("open", &["open"][..]),
        ("popup_open", &["open", "popup-open", "popup_open"][..]),
        ("disabled", &["disabled"][..]),
        ("dragging", &["dragging"][..]),
    ] {
        if bool_attribute(node, attribute) {
            for alias in aliases {
                append_state(&mut states, alias);
            }
        }
    }
    if bool_attribute(node, "error")
        || string_attribute_any(node, &["validation_level"])
            .is_some_and(|value| matches!(value.as_str(), "error" | "danger"))
    {
        append_state(&mut states, "error");
    }
    states
}

fn mui_state_classes(node: &UiTemplateNode) -> Vec<String> {
    let mut classes = Vec::new();
    for (attribute, class_name) in [
        ("disabled", "Mui-disabled"),
        ("focused", "Mui-focused"),
        ("selected", "Mui-selected"),
        ("checked", "Mui-checked"),
        ("expanded", "Mui-expanded"),
        ("hovered", "Mui-hovered"),
        ("pressed", "Mui-pressed"),
        ("dragging", "Mui-dragging"),
    ] {
        if bool_attribute(node, attribute) {
            append_class(&mut classes, class_name.to_string());
        }
    }
    if bool_attribute(node, "open") || bool_attribute(node, "popup_open") {
        append_class(&mut classes, "Mui-open".to_string());
    }
    if bool_attribute(node, "active") {
        append_class(&mut classes, "Mui-active".to_string());
    }
    if bool_attribute(node, "completed") {
        append_class(&mut classes, "Mui-completed".to_string());
    }
    if bool_attribute_any(node, &["focusVisible", "focus_visible"]) {
        append_class(&mut classes, "Mui-focusVisible".to_string());
    }
    if bool_attribute_any(
        node,
        &["read_only", "readOnly", "input_read_only", "inputReadOnly"],
    ) {
        append_class(&mut classes, "Mui-readOnly".to_string());
    }
    if bool_attribute(node, "error")
        || string_attribute_any(node, &["validation_level"])
            .is_some_and(|value| matches!(value.as_str(), "error" | "danger"))
    {
        append_class(&mut classes, "Mui-error".to_string());
    }
    classes
}

fn string_attribute_any(node: &UiTemplateNode, names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        node.attributes
            .get(*name)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn mui_slot_name(node: &UiTemplateNode) -> Option<String> {
    string_from_map(&node.slot_attributes, "mui_slot")
        .or_else(|| string_from_map(&node.attributes, "mui_slot"))
}

fn string_from_map(values: &BTreeMap<String, Value>, name: &str) -> Option<String> {
    values
        .get(name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn string_from_attributes_any(
    attributes: &BTreeMap<String, Value>,
    names: &[&str],
) -> Option<String> {
    names
        .iter()
        .find_map(|name| string_from_map(attributes, name))
}

fn bool_from_attributes_any(attributes: &BTreeMap<String, Value>, names: &[&str]) -> bool {
    names.iter().any(|name| {
        attributes
            .get(*name)
            .and_then(Value::as_bool)
            .unwrap_or(false)
    })
}

fn nested_map_attribute(
    node: &UiTemplateNode,
    attribute: &str,
    nested: &str,
) -> Option<BTreeMap<String, Value>> {
    node.attributes
        .get(attribute)
        .and_then(Value::as_table)
        .and_then(|table| table.get(nested))
        .and_then(value_as_map)
}

fn nested_map_attribute_any(
    node: &UiTemplateNode,
    attributes: &[&str],
    nested: &str,
) -> BTreeMap<String, Value> {
    let mut merged = BTreeMap::new();
    for attribute in attributes {
        if let Some(value) = nested_map_attribute(node, attribute, nested) {
            merge_value_maps(&mut merged, &value);
        }
    }
    merged
}

fn map_attribute(node: &UiTemplateNode, attribute: &str) -> BTreeMap<String, Value> {
    node.attributes
        .get(attribute)
        .and_then(value_as_map)
        .unwrap_or_default()
}

fn map_from_attributes(
    attributes: &BTreeMap<String, Value>,
    attribute: &str,
) -> BTreeMap<String, Value> {
    attributes
        .get(attribute)
        .and_then(value_as_map)
        .unwrap_or_default()
}

fn map_attribute_any(node: &UiTemplateNode, attributes: &[&str]) -> BTreeMap<String, Value> {
    let mut merged = BTreeMap::new();
    for attribute in attributes {
        merge_value_maps(&mut merged, &map_attribute(node, attribute));
    }
    merged
}

fn value_as_map(value: &Value) -> Option<BTreeMap<String, Value>> {
    value.as_table().map(|table| {
        table
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    })
}

fn string_attribute_any_prefer_non_default(
    node: &UiTemplateNode,
    names: &[(&str, &str)],
) -> Option<String> {
    // Descriptor defaults make both MUI and legacy props present, so prefer the first value that
    // differs from its default while keeping the first default as the fallback class.
    let mut first = None;
    for (name, default) in names {
        let Some(value) = node.attributes.get(*name).and_then(Value::as_str) else {
            continue;
        };
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        first.get_or_insert_with(|| trimmed.to_string());
        if trimmed != *default {
            return Some(trimmed.to_string());
        }
    }
    first
}

fn string_array_attribute(node: &UiTemplateNode, name: &str) -> Vec<String> {
    let Some(values) = node.attributes.get(name).and_then(Value::as_array) else {
        return Vec::new();
    };
    values
        .iter()
        .filter_map(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .collect()
}

fn string_list_attribute(node: &UiTemplateNode, name: &str) -> Vec<String> {
    string_list_from_value(node.attributes.get(name))
}

fn classes_map_slot_classes(node: &UiTemplateNode, slot: &str) -> Vec<String> {
    node.attributes
        .get("classes")
        .and_then(Value::as_table)
        .and_then(|table| table.get(slot))
        .map(|value| string_list_from_value(Some(value)))
        .unwrap_or_default()
}

fn string_list_from_value(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(value)) => split_class_names(value),
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(Value::as_str)
            .flat_map(split_class_names)
            .collect(),
        _ => Vec::new(),
    }
}

fn split_class_names(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn bool_attribute(node: &UiTemplateNode, name: &str) -> bool {
    node.attributes
        .get(name)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn bool_attribute_any(node: &UiTemplateNode, names: &[&str]) -> bool {
    names.iter().any(|name| bool_attribute(node, name))
}

fn int_attribute_any(node: &UiTemplateNode, names: &[&str]) -> Option<i32> {
    names.iter().find_map(|name| {
        node.attributes.get(*name).and_then(|value| match value {
            Value::Integer(value) => i32::try_from(*value).ok(),
            Value::Float(value) => value.is_finite().then_some(value.round() as i32),
            Value::String(value) => value.trim().parse().ok(),
            _ => None,
        })
    })
}

fn append_class(classes: &mut Vec<String>, class_name: String) {
    if !classes.iter().any(|value| value == &class_name) {
        classes.push(class_name);
    }
}

fn append_state(states: &mut Vec<String>, state: &str) {
    if !states.iter().any(|value| value == state) {
        states.push(state.to_string());
    }
}

fn pascal_case(value: &str) -> String {
    value
        .split(['-', '_', ' '])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<String>()
}
