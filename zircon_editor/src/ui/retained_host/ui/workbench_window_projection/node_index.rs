use std::collections::{HashMap, HashSet};

use crate::ui::template_runtime::{RetainedUiHostNodeModel, RetainedUiHostValue};

pub(super) struct ProjectionNodeIndex<'a> {
    nodes_by_id: HashMap<&'a str, &'a RetainedUiHostNodeModel>,
    render_visible_by_id: HashMap<&'a str, bool>,
    nearest_control_node_id_by_id: HashMap<&'a str, Option<&'a str>>,
}

impl<'a> ProjectionNodeIndex<'a> {
    pub(super) fn new(nodes: impl IntoIterator<Item = &'a RetainedUiHostNodeModel>) -> Self {
        let nodes_by_id = nodes
            .into_iter()
            .map(|node| (node.node_id.as_str(), node))
            .collect::<HashMap<_, _>>();
        let render_visible_by_id = build_render_visibility(&nodes_by_id);
        let nearest_control_node_id_by_id = build_nearest_control_nodes(&nodes_by_id);
        Self {
            nodes_by_id,
            render_visible_by_id,
            nearest_control_node_id_by_id,
        }
    }

    pub(super) fn node(&self, node_id: &str) -> Option<&'a RetainedUiHostNodeModel> {
        self.nodes_by_id.get(node_id).copied()
    }

    pub(super) fn node_count(&self) -> usize {
        self.nodes_by_id.len()
    }

    pub(super) fn render_visible(&self, node: &RetainedUiHostNodeModel) -> bool {
        self.render_visible_by_id
            .get(node.node_id.as_str())
            .copied()
            .unwrap_or_else(|| node_properties_render_visible(node))
    }

    pub(super) fn projected_parent_node_id(
        &self,
        node: &RetainedUiHostNodeModel,
    ) -> Option<&'a str> {
        let parent_id = node.parent_id.as_deref()?;
        self.nearest_control_node_id_by_id
            .get(parent_id)
            .copied()
            .flatten()
    }
}

fn build_render_visibility<'a>(
    nodes_by_id: &HashMap<&'a str, &'a RetainedUiHostNodeModel>,
) -> HashMap<&'a str, bool> {
    let mut visibility = HashMap::with_capacity(nodes_by_id.len());
    let mut path = Vec::new();
    let mut visiting = HashSet::new();

    for node in nodes_by_id.values().copied() {
        if visibility.contains_key(node.node_id.as_str()) {
            continue;
        }
        path.clear();
        visiting.clear();
        let mut current = Some(node);
        let inherited_visible = loop {
            let Some(current_node) = current else {
                break true;
            };
            let current_id = current_node.node_id.as_str();
            if let Some(cached) = visibility.get(current_id) {
                break *cached;
            }
            if !visiting.insert(current_id) {
                break false;
            }
            let local_visible = node_properties_render_visible(current_node);
            path.push((current_id, local_visible));
            if !local_visible {
                break false;
            }
            current = current_node
                .parent_id
                .as_deref()
                .and_then(|parent_id| nodes_by_id.get(parent_id).copied());
        };

        let mut visible = inherited_visible;
        while let Some((node_id, local_visible)) = path.pop() {
            visible &= local_visible;
            visibility.insert(node_id, visible);
        }
    }

    visibility
}

fn build_nearest_control_nodes<'a>(
    nodes_by_id: &HashMap<&'a str, &'a RetainedUiHostNodeModel>,
) -> HashMap<&'a str, Option<&'a str>> {
    let mut nearest = HashMap::with_capacity(nodes_by_id.len());
    let mut path = Vec::new();
    let mut visiting = HashSet::new();

    for node in nodes_by_id.values().copied() {
        if nearest.contains_key(node.node_id.as_str()) {
            continue;
        }
        path.clear();
        visiting.clear();
        let mut current = Some(node);
        let resolved = loop {
            let Some(current_node) = current else {
                break None;
            };
            let current_id = current_node.node_id.as_str();
            if let Some(cached) = nearest.get(current_id) {
                break *cached;
            }
            if !visiting.insert(current_id) {
                break None;
            }
            if current_node.control_id.is_some() {
                nearest.insert(current_id, Some(current_id));
                break Some(current_id);
            }
            path.push(current_id);
            current = current_node
                .parent_id
                .as_deref()
                .and_then(|parent_id| nodes_by_id.get(parent_id).copied());
        };

        while let Some(node_id) = path.pop() {
            nearest.insert(node_id, resolved);
        }
    }

    nearest
}

fn node_properties_render_visible(node: &RetainedUiHostNodeModel) -> bool {
    let visibility = node.properties.get("visibility");
    if !legacy_visible_property(node) && !visibility_is(visibility, "collapsed") {
        return false;
    }
    visibility.is_none()
        || visibility_is(visibility, "visible")
        || visibility_is(visibility, "hittestinvisible")
        || visibility_is(visibility, "selfhittestinvisible")
}

fn legacy_visible_property(node: &RetainedUiHostNodeModel) -> bool {
    match node.properties.get("visible") {
        Some(RetainedUiHostValue::Bool(value)) => *value,
        Some(RetainedUiHostValue::String(value)) => value.parse().unwrap_or(true),
        _ => true,
    }
}

fn visibility_is(value: Option<&RetainedUiHostValue>, expected: &str) -> bool {
    let Some(value) = value else {
        return expected == "visible";
    };
    match value {
        RetainedUiHostValue::String(value) => normalized_ascii_eq(value, expected),
        RetainedUiHostValue::Integer(value) => normalized_ascii_eq(&value.to_string(), expected),
        RetainedUiHostValue::Float(value) => normalized_ascii_eq(&value.to_string(), expected),
        RetainedUiHostValue::Bool(value) => normalized_ascii_eq(&value.to_string(), expected),
        RetainedUiHostValue::Datetime(_)
        | RetainedUiHostValue::Array(_)
        | RetainedUiHostValue::Table(_) => expected == "visible",
    }
}

fn normalized_ascii_eq(value: &str, expected: &str) -> bool {
    value
        .bytes()
        .filter(|byte| *byte != b'_')
        .map(|byte| byte.to_ascii_lowercase())
        .eq(expected.bytes())
}
