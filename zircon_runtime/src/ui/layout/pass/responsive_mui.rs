use toml::Value;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{
        UiContainerKind, UiGridBoxConfig, UiGridSlotPlacement, UiLinearBoxConfig,
        UiMasonryBoxConfig, UiSize, UiSlotKind,
    },
    tree::{UiTree, UiTreeError, UiVisibility},
};

const MUI_DEFAULT_SPACING_UNIT: f32 = 8.0;

// Mirrors Material UI's default breakpoint keys so authored responsive props
// resolve before measurement and feed one stable Zircon/Taffy layout contract.
const BREAKPOINTS: [(&str, f32); 5] = [
    ("xs", 0.0),
    ("sm", 600.0),
    ("md", 900.0),
    ("lg", 1200.0),
    ("xl", 1536.0),
];

pub(super) fn apply_mui_responsive_layout(
    tree: &mut UiTree,
    root_size: UiSize,
) -> Result<(), UiTreeError> {
    let viewport = MuiResponsiveViewport::from_root_size(root_size);
    apply_use_media_query_matches(tree, viewport)?;
    apply_responsive_visibility(tree, viewport)?;
    apply_responsive_containers(tree, viewport)?;
    apply_responsive_grid_slots(tree, viewport)
}

fn apply_use_media_query_matches(
    tree: &mut UiTree,
    viewport: MuiResponsiveViewport,
) -> Result<(), UiTreeError> {
    let node_ids = tree.nodes.keys().copied().collect::<Vec<_>>();
    for node_id in node_ids {
        let Some(next_matches) = use_media_query_match_for_node(tree, node_id, viewport)? else {
            continue;
        };
        let node = tree
            .nodes
            .get_mut(&node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_mut() else {
            continue;
        };
        if metadata.attributes.get("matches").and_then(Value::as_bool) == Some(next_matches) {
            continue;
        }
        metadata
            .attributes
            .insert("matches".to_string(), Value::Boolean(next_matches));
        mark_node_layout_dirty(node);
        node.dirty.input = true;
    }
    Ok(())
}

fn apply_responsive_visibility(
    tree: &mut UiTree,
    viewport: MuiResponsiveViewport,
) -> Result<(), UiTreeError> {
    let node_ids = tree.nodes.keys().copied().collect::<Vec<_>>();
    for node_id in node_ids {
        let Some(next) = responsive_visibility_for_node(tree, node_id, viewport)? else {
            continue;
        };
        let node = tree
            .nodes
            .get_mut(&node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if node.visibility == next.visibility && node.state_flags.visible == next.legacy_visible {
            continue;
        }
        let previous_effective = node.visibility.effective(node.state_flags.visible);
        let next_effective = next.visibility.effective(next.legacy_visible);
        node.visibility = next.visibility;
        node.state_flags.visible = next.legacy_visible;
        node.state_flags.dirty = true;
        mark_node_visibility_dirty(
            node,
            previous_effective.occupies_layout() != next_effective.occupies_layout(),
        );
    }
    Ok(())
}

fn apply_responsive_containers(
    tree: &mut UiTree,
    viewport: MuiResponsiveViewport,
) -> Result<(), UiTreeError> {
    let node_ids = tree.nodes.keys().copied().collect::<Vec<_>>();
    for node_id in node_ids {
        let Some(container) = responsive_container_for_node(tree, node_id, viewport)? else {
            continue;
        };
        let node = tree
            .nodes
            .get_mut(&node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if node.container != container {
            node.container = container;
            mark_node_layout_dirty(node);
        }
    }
    Ok(())
}

fn apply_responsive_grid_slots(
    tree: &mut UiTree,
    viewport: MuiResponsiveViewport,
) -> Result<(), UiTreeError> {
    for index in 0..tree.slots.len() {
        let (parent_id, child_id) = {
            let slot = &tree.slots[index];
            (slot.parent_id, slot.child_id)
        };
        if !is_implicit_mui_grid_container(tree, parent_id)? {
            continue;
        }
        if child_has_explicit_slot_layout(tree, child_id)? {
            continue;
        }
        let placement = responsive_grid_item_placement(tree, child_id, viewport)?;
        let slot = &mut tree.slots[index];
        if slot.kind != UiSlotKind::Grid || slot.grid_placement != placement {
            slot.kind = UiSlotKind::Grid;
            slot.grid_placement = placement;
            mark_pair_layout_dirty(tree, parent_id, child_id);
        }
    }
    Ok(())
}

fn is_implicit_mui_grid_container(tree: &UiTree, node_id: UiNodeId) -> Result<bool, UiTreeError> {
    let node = tree
        .nodes
        .get(&node_id)
        .ok_or(UiTreeError::MissingParent(node_id))?;
    if !matches!(node.container, UiContainerKind::GridBox(_)) {
        return Ok(false);
    }
    let Some(metadata) = node.template_metadata.as_ref() else {
        return Ok(false);
    };
    Ok(metadata.component == "Grid"
        && bool_attribute(&metadata.attributes, "container")
        && !has_explicit_layout_container(&metadata.attributes))
}

fn responsive_container_for_node(
    tree: &UiTree,
    node_id: UiNodeId,
    viewport: MuiResponsiveViewport,
) -> Result<Option<UiContainerKind>, UiTreeError> {
    let node = tree
        .nodes
        .get(&node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    let Some(metadata) = node.template_metadata.as_ref() else {
        return Ok(None);
    };
    if has_explicit_layout_container(&metadata.attributes) {
        return Ok(None);
    }
    Ok(match metadata.component.as_str() {
        "Grid" if bool_attribute(&metadata.attributes, "container") => Some(
            UiContainerKind::GridBox(mui_grid_config(&metadata.attributes, viewport)),
        ),
        "Stack" => Some(mui_stack_container(&metadata.attributes, viewport)),
        "Masonry" => Some(UiContainerKind::MasonryBox(mui_masonry_config(
            &metadata.attributes,
            viewport,
        ))),
        _ => None,
    })
}

fn responsive_grid_item_placement(
    tree: &UiTree,
    child_id: UiNodeId,
    viewport: MuiResponsiveViewport,
) -> Result<Option<UiGridSlotPlacement>, UiTreeError> {
    let child = tree
        .nodes
        .get(&child_id)
        .ok_or(UiTreeError::MissingNode(child_id))?;
    let Some(metadata) = child.template_metadata.as_ref() else {
        return Ok(None);
    };
    let Some(span) = responsive_usize_attribute(&metadata.attributes, &["size"], viewport) else {
        return Ok(None);
    };
    let column =
        responsive_usize_attribute(&metadata.attributes, &["offset"], viewport).unwrap_or(0);
    Ok(Some(UiGridSlotPlacement::new(column, 0).with_span(span, 1)))
}

fn responsive_visibility_for_node(
    tree: &UiTree,
    node_id: UiNodeId,
    viewport: MuiResponsiveViewport,
) -> Result<Option<ResponsiveVisibility>, UiTreeError> {
    let node = tree
        .nodes
        .get(&node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    let Some(metadata) = node.template_metadata.as_ref() else {
        return Ok(None);
    };
    let display = responsive_display_attribute(&metadata.attributes, viewport);
    let visibility = responsive_visibility_attribute(&metadata.attributes, viewport);
    let legacy_visible = responsive_bool_attribute(&metadata.attributes, &["visible"], viewport);
    if display.is_none() && visibility.is_none() && legacy_visible.is_none() {
        return Ok(None);
    }

    let mut resolved_visibility = visibility.unwrap_or_else(|| {
        if matches!(display, Some(ResponsiveDisplay::Shown)) {
            UiVisibility::Visible
        } else {
            node.visibility
        }
    });
    let resolved_legacy_visible = legacy_visible.unwrap_or(node.state_flags.visible);
    if matches!(display, Some(ResponsiveDisplay::None)) {
        resolved_visibility = UiVisibility::Collapsed;
    }

    Ok(Some(ResponsiveVisibility {
        visibility: resolved_visibility,
        legacy_visible: resolved_legacy_visible,
    }))
}

fn use_media_query_match_for_node(
    tree: &UiTree,
    node_id: UiNodeId,
    viewport: MuiResponsiveViewport,
) -> Result<Option<bool>, UiTreeError> {
    let node = tree
        .nodes
        .get(&node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    let Some(metadata) = node.template_metadata.as_ref() else {
        return Ok(None);
    };
    if metadata.component != "UseMediaQuery" {
        return Ok(None);
    }

    Ok(Some(
        query_match(&metadata.attributes, viewport)
            .or_else(|| breakpoint_query_match(&metadata.attributes, viewport))
            .or_else(|| {
                width_floor_attribute(&metadata.attributes, &["min_width", "minWidth"])
                    .map(|min_width| viewport.width >= min_width)
            })
            .or_else(|| {
                width_floor_attribute(&metadata.attributes, &["max_width", "maxWidth"])
                    .map(|max_width| viewport.width <= max_width)
            })
            .unwrap_or_else(|| fallback_media_query_match(&metadata.attributes)),
    ))
}

fn breakpoint_query_match(
    attributes: &std::collections::BTreeMap<String, Value>,
    viewport: MuiResponsiveViewport,
) -> Option<bool> {
    breakpoint_width_attribute(attributes, &["up", "breakpoint"])
        .map(|min_width| viewport.width >= min_width)
        .or_else(|| {
            breakpoint_width_attribute(attributes, &["down"])
                .map(|max_exclusive| viewport.width < max_exclusive)
        })
        .or_else(|| {
            breakpoint_range_attribute(attributes, "between").map(|(min_width, max_exclusive)| {
                viewport.width >= min_width && viewport.width < max_exclusive
            })
        })
}

fn query_match(
    attributes: &std::collections::BTreeMap<String, Value>,
    viewport: MuiResponsiveViewport,
) -> Option<bool> {
    let query = attributes.get("query")?.as_str()?.trim();
    let normalized = normalize_media_query(query);
    let min_width = width_query_threshold(&normalized, "minwidth:");
    let max_width = width_query_threshold(&normalized, "maxwidth:");
    if min_width.is_none() && max_width.is_none() {
        return None;
    }
    Some(
        min_width.is_none_or(|min_width| viewport.width >= min_width)
            && max_width.is_none_or(|max_width| viewport.width <= max_width),
    )
}

fn width_query_threshold(normalized_query: &str, prefix: &str) -> Option<f32> {
    let start = normalized_query.find(prefix)? + prefix.len();
    let width = &normalized_query[start..];
    let width = width
        .find("px")
        .map(|index| {
            let (prefix, _) = width.split_at(index);
            prefix
        })
        .filter(|value| !value.is_empty())?;
    width.parse::<f32>().ok().filter(|value| value.is_finite())
}

fn normalize_media_query(query: &str) -> String {
    query
        .chars()
        .filter(|ch| !matches!(*ch, '(' | ')' | ' ' | '-'))
        .flat_map(char::to_lowercase)
        .collect()
}

fn width_floor_attribute(
    attributes: &std::collections::BTreeMap<String, Value>,
    names: &[&str],
) -> Option<f32> {
    names
        .iter()
        .find_map(|name| attributes.get(*name))
        .and_then(value_as_plain_f32)
}

fn breakpoint_width_attribute(
    attributes: &std::collections::BTreeMap<String, Value>,
    names: &[&str],
) -> Option<f32> {
    names
        .iter()
        .find_map(|name| attributes.get(*name))
        .and_then(value_as_breakpoint_width)
}

fn breakpoint_range_attribute(
    attributes: &std::collections::BTreeMap<String, Value>,
    name: &str,
) -> Option<(f32, f32)> {
    let values = attributes.get(name)?.as_array()?;
    let start = values.first().and_then(value_as_breakpoint_width)?;
    let end = values.get(1).and_then(value_as_breakpoint_width)?;
    (start < end).then_some((start, end))
}

fn fallback_media_query_match(attributes: &std::collections::BTreeMap<String, Value>) -> bool {
    bool_attribute_any(attributes, &["defaultMatches", "default_matches"])
        .or_else(|| bool_attribute_any(attributes, &["matches"]))
        .unwrap_or(false)
}

fn mui_grid_config(
    attributes: &std::collections::BTreeMap<String, Value>,
    viewport: MuiResponsiveViewport,
) -> UiGridBoxConfig {
    let spacing = responsive_f32_attribute(attributes, &["spacing"], viewport).unwrap_or(0.0);
    UiGridBoxConfig {
        columns: responsive_usize_attribute(attributes, &["columns"], viewport)
            .unwrap_or(12)
            .max(1),
        rows: 1,
        column_gap: responsive_f32_attribute(
            attributes,
            &["columnSpacing", "column_spacing"],
            viewport,
        )
        .unwrap_or(spacing)
        .max(0.0),
        row_gap: responsive_f32_attribute(attributes, &["rowSpacing", "row_spacing"], viewport)
            .unwrap_or(spacing)
            .max(0.0),
    }
}

fn mui_stack_container(
    attributes: &std::collections::BTreeMap<String, Value>,
    viewport: MuiResponsiveViewport,
) -> UiContainerKind {
    let gap = responsive_f32_attribute(attributes, &["spacing"], viewport)
        .unwrap_or(0.0)
        .max(0.0);
    let config = UiLinearBoxConfig { gap };
    match responsive_string_attribute(attributes, &["direction"], viewport).as_deref() {
        Some("row") | Some("row-reverse") => UiContainerKind::HorizontalBox(config),
        _ => UiContainerKind::VerticalBox(config),
    }
}

fn mui_masonry_config(
    attributes: &std::collections::BTreeMap<String, Value>,
    viewport: MuiResponsiveViewport,
) -> UiMasonryBoxConfig {
    UiMasonryBoxConfig {
        columns: responsive_usize_attribute(attributes, &["columns"], viewport)
            .unwrap_or(UiMasonryBoxConfig::default().columns)
            .max(1),
        gap: responsive_f32_attribute(attributes, &["spacing"], viewport)
            .unwrap_or(0.0)
            .max(0.0),
        sequential: bool_attribute(attributes, "sequential"),
    }
}

fn has_explicit_layout_container(attributes: &std::collections::BTreeMap<String, Value>) -> bool {
    attributes
        .get("layout")
        .and_then(Value::as_table)
        .is_some_and(|layout| layout.contains_key("container"))
}

fn child_has_explicit_slot_layout(tree: &UiTree, child_id: UiNodeId) -> Result<bool, UiTreeError> {
    let child = tree
        .nodes
        .get(&child_id)
        .ok_or(UiTreeError::MissingNode(child_id))?;
    Ok(child
        .template_metadata
        .as_ref()
        .is_some_and(|metadata| metadata.slot_attributes.contains_key("layout")))
}

fn bool_attribute(attributes: &std::collections::BTreeMap<String, Value>, name: &str) -> bool {
    attributes
        .get(name)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn bool_attribute_any(
    attributes: &std::collections::BTreeMap<String, Value>,
    names: &[&str],
) -> Option<bool> {
    names
        .iter()
        .find_map(|name| attributes.get(*name))
        .and_then(value_as_bool)
}

fn responsive_usize_attribute(
    attributes: &std::collections::BTreeMap<String, Value>,
    names: &[&str],
    viewport: MuiResponsiveViewport,
) -> Option<usize> {
    names
        .iter()
        .find_map(|name| viewport.responsive_value(attributes.get(*name)))
        .and_then(value_as_usize)
}

fn responsive_f32_attribute(
    attributes: &std::collections::BTreeMap<String, Value>,
    names: &[&str],
    viewport: MuiResponsiveViewport,
) -> Option<f32> {
    names
        .iter()
        .find_map(|name| viewport.responsive_value(attributes.get(*name)))
        .and_then(value_as_f32)
}

fn responsive_bool_attribute(
    attributes: &std::collections::BTreeMap<String, Value>,
    names: &[&str],
    viewport: MuiResponsiveViewport,
) -> Option<bool> {
    names
        .iter()
        .find_map(|name| viewport.responsive_value(attributes.get(*name)))
        .and_then(value_as_bool)
}

fn responsive_string_attribute(
    attributes: &std::collections::BTreeMap<String, Value>,
    names: &[&str],
    viewport: MuiResponsiveViewport,
) -> Option<String> {
    names
        .iter()
        .find_map(|name| viewport.responsive_value(attributes.get(*name)))
        .and_then(value_as_string)
}

fn responsive_display_attribute(
    attributes: &std::collections::BTreeMap<String, Value>,
    viewport: MuiResponsiveViewport,
) -> Option<ResponsiveDisplay> {
    let value = viewport.responsive_value(attributes.get("display"))?;
    match value {
        Value::Boolean(false) => Some(ResponsiveDisplay::None),
        Value::Boolean(true) => Some(ResponsiveDisplay::Shown),
        Value::String(value) if value.trim().eq_ignore_ascii_case("none") => {
            Some(ResponsiveDisplay::None)
        }
        Value::String(value) if !value.trim().is_empty() => Some(ResponsiveDisplay::Shown),
        _ => None,
    }
}

fn responsive_visibility_attribute(
    attributes: &std::collections::BTreeMap<String, Value>,
    viewport: MuiResponsiveViewport,
) -> Option<UiVisibility> {
    let value = viewport.responsive_value(attributes.get("visibility"))?;
    value_as_visibility(value)
}

fn value_as_usize(value: &Value) -> Option<usize> {
    match value {
        Value::Integer(value) => usize::try_from(*value).ok(),
        Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        Value::String(value) => value.trim().parse().ok(),
        _ => None,
    }
}

fn value_as_f32(value: &Value) -> Option<f32> {
    match value {
        Value::Integer(value) => Some(*value as f32 * MUI_DEFAULT_SPACING_UNIT),
        Value::Float(value) if value.is_finite() => Some(*value as f32 * MUI_DEFAULT_SPACING_UNIT),
        Value::String(value) => value.trim().parse().ok(),
        _ => None,
    }
}

fn value_as_plain_f32(value: &Value) -> Option<f32> {
    match value {
        Value::Integer(value) => Some(*value as f32),
        Value::Float(value) if value.is_finite() => Some(*value as f32),
        Value::String(value) => value.trim().trim_end_matches("px").parse().ok(),
        _ => None,
    }
}

fn value_as_breakpoint_width(value: &Value) -> Option<f32> {
    value_as_string(value)
        .as_deref()
        .and_then(breakpoint_min_width)
        .or_else(|| value_as_plain_f32(value))
}

fn breakpoint_min_width(value: &str) -> Option<f32> {
    let token = normalized_token(value);
    BREAKPOINTS
        .iter()
        .find_map(|(breakpoint, min_width)| (*breakpoint == token).then_some(*min_width))
}

fn value_as_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Boolean(value) => Some(*value),
        Value::String(value) => match normalized_token(value).as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn value_as_string(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn value_as_visibility(value: &Value) -> Option<UiVisibility> {
    match value {
        Value::String(value) => match normalized_token(value).as_str() {
            "visible" => Some(UiVisibility::Visible),
            "hidden" => Some(UiVisibility::Hidden),
            "collapsed" | "collapse" => Some(UiVisibility::Collapsed),
            "hittestinvisible" => Some(UiVisibility::HitTestInvisible),
            "selfhittestinvisible" => Some(UiVisibility::SelfHitTestInvisible),
            _ => None,
        },
        Value::Boolean(true) => Some(UiVisibility::Visible),
        Value::Boolean(false) => Some(UiVisibility::Hidden),
        _ => None,
    }
}

fn normalized_token(value: &str) -> String {
    value
        .chars()
        .filter(|ch| *ch != '_' && *ch != '-' && !ch.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

fn mark_pair_layout_dirty(tree: &mut UiTree, parent_id: UiNodeId, child_id: UiNodeId) {
    if let Some(parent) = tree.nodes.get_mut(&parent_id) {
        mark_node_layout_dirty(parent);
    }
    if let Some(child) = tree.nodes.get_mut(&child_id) {
        mark_node_layout_dirty(child);
    }
}

fn mark_node_layout_dirty(node: &mut zircon_runtime_interface::ui::tree::UiTreeNode) {
    node.dirty.layout = true;
    node.dirty.hit_test = true;
    node.dirty.render = true;
    node.dirty.visible_range |= matches!(
        node.container,
        UiContainerKind::ScrollableBox(_) | UiContainerKind::MasonryBox(_)
    );
}

fn mark_node_visibility_dirty(
    node: &mut zircon_runtime_interface::ui::tree::UiTreeNode,
    layout_changed: bool,
) {
    node.dirty.layout |= layout_changed;
    node.dirty.hit_test = true;
    node.dirty.render = true;
    node.dirty.input = true;
    node.dirty.visible_range |= matches!(
        node.container,
        UiContainerKind::ScrollableBox(_) | UiContainerKind::MasonryBox(_)
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ResponsiveVisibility {
    visibility: UiVisibility,
    legacy_visible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResponsiveDisplay {
    None,
    Shown,
}

#[derive(Clone, Copy, Debug)]
struct MuiResponsiveViewport {
    width: f32,
}

impl MuiResponsiveViewport {
    fn from_root_size(root_size: UiSize) -> Self {
        Self {
            width: root_size
                .width
                .is_finite()
                .then_some(root_size.width.max(0.0))
                .unwrap_or(0.0),
        }
    }

    fn responsive_value<'a>(&self, value: Option<&'a Value>) -> Option<&'a Value> {
        match value? {
            Value::Table(values) => {
                let mut selected = None;
                for (breakpoint, min_width) in BREAKPOINTS {
                    if self.width >= min_width {
                        if let Some(value) = values.get(breakpoint) {
                            selected = Some(value);
                        }
                    }
                }
                selected
            }
            Value::Array(values) => {
                let mut selected = None;
                for (index, value) in values.iter().enumerate() {
                    let Some((_, min_width)) = BREAKPOINTS.get(index).copied() else {
                        break;
                    };
                    if self.width >= min_width {
                        selected = Some(value);
                    }
                }
                selected
            }
            scalar => Some(scalar),
        }
    }
}
