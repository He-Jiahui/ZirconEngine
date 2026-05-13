use std::collections::{BTreeMap, BTreeSet};

use toml::Value;
use zircon_runtime_interface::ui::component::UiComponentState;
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::template::{
    UiSelector, UiSelectorCombinator, UiSelectorSegment, UiSelectorToken,
};
use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTree, UiTreeError, UiTreeNode};
use zircon_runtime_interface::ui::v2::{
    UiV2AssetDocument, UiV2AssetError, UiV2NodeArena, UiV2NodeHandle, UiV2ResolvedStyle,
    UiV2ResolvedStyleSheet, UiV2StyleDeclarationBlock,
};

#[derive(Default)]
pub struct UiV2StyleResolver;

impl UiV2StyleResolver {
    pub fn resolve(
        document: &UiV2AssetDocument,
        arena: &UiV2NodeArena,
    ) -> Result<UiV2ResolvedStyleSheet, UiV2AssetError> {
        let rules = collect_rules(document)?;
        Self::resolve_with_rules(document, arena, &rules)
    }

    pub(crate) fn resolve_static(
        document: &UiV2AssetDocument,
        arena: &UiV2NodeArena,
    ) -> Result<UiV2ResolvedStyleSheet, UiV2AssetError> {
        let rules = collect_rules(document)?
            .into_iter()
            .filter(|rule| !rule.uses_pseudo_state())
            .collect::<Vec<_>>();
        Self::resolve_with_rules(document, arena, &rules)
    }

    fn resolve_with_rules(
        document: &UiV2AssetDocument,
        arena: &UiV2NodeArena,
        rules: &[ResolvedRule],
    ) -> Result<UiV2ResolvedStyleSheet, UiV2AssetError> {
        let mut resolved = UiV2ResolvedStyleSheet::default();
        let Some(root) = arena.root else {
            return Ok(resolved);
        };

        let mut path = Vec::new();
        let mut stack = vec![StyleFrame::new(root)];
        while let Some(frame) = stack.last_mut() {
            if !frame.entered {
                let node = arena
                    .node(frame.handle)
                    .ok_or_else(|| UiV2AssetError::MissingNode {
                        asset_id: document.asset.id.clone(),
                        node_id: format!("handle {}", frame.handle.index()),
                    })?;
                path.push(SelectorPathNode::from_arena_node(node, path.is_empty()));
                let mut node_style = UiV2ResolvedStyle::default();
                for rule in rules {
                    if rule.selector.matches_path(&path) {
                        node_style.merge_block(&rule.set);
                    }
                }
                node_style.merge_block(&node.style);
                resolve_value_map(&mut node_style.self_values, &document.tokens, 0);
                resolve_value_map(&mut node_style.slot, &document.tokens, 0);
                let _ = resolved.nodes.insert(node.source_id.clone(), node_style);
                frame.entered = true;
            }

            let node = arena
                .node(frame.handle)
                .expect("style traversal only pushes handles from arena nodes");
            if frame.next_child < node.children.len() {
                let child = node.children[frame.next_child].child;
                frame.next_child += 1;
                stack.push(StyleFrame::new(child));
            } else {
                let _ = stack.pop();
                let _ = path.pop();
            }
        }

        Ok(resolved)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct UiV2RuntimeStyleIndex {
    rules: Vec<ResolvedRule>,
    base_attributes: BTreeMap<UiNodeId, BTreeMap<String, Value>>,
    base_style_overrides: BTreeMap<UiNodeId, BTreeMap<String, Value>>,
}

impl UiV2RuntimeStyleIndex {
    pub(crate) fn from_document(document: &UiV2AssetDocument) -> Result<Self, UiV2AssetError> {
        let mut rules = collect_rules(document)?
            .into_iter()
            .filter(ResolvedRule::uses_pseudo_state)
            .collect::<Vec<_>>();
        for rule in &mut rules {
            resolve_value_map(&mut rule.set.self_values, &document.tokens, 0);
            resolve_value_map(&mut rule.set.slot, &document.tokens, 0);
        }
        Ok(Self {
            rules,
            base_attributes: BTreeMap::new(),
            base_style_overrides: BTreeMap::new(),
        })
    }

    pub(crate) fn has_runtime_rules(&self) -> bool {
        !self.rules.is_empty()
    }

    pub(crate) fn capture_baseline_from_tree(&mut self, tree: &UiTree) {
        self.base_attributes.clear();
        self.base_style_overrides.clear();
        for (node_id, node) in &tree.nodes {
            let Some(metadata) = node.template_metadata.as_ref() else {
                continue;
            };
            let _ = self
                .base_attributes
                .insert(*node_id, metadata.attributes.clone());
            let _ = self
                .base_style_overrides
                .insert(*node_id, metadata.style_overrides.clone());
        }
    }

    pub(crate) fn apply_to_tree_subtree(
        &self,
        tree: &mut UiTree,
        component_states: &crate::ui::surface::UiSurfaceComponentStateStore,
        root_id: UiNodeId,
        mark_dirty: bool,
    ) -> Result<usize, UiTreeError> {
        if self.rules.is_empty() {
            return Ok(0);
        }
        if !tree.nodes.contains_key(&root_id) {
            return Err(UiTreeError::MissingNode(root_id));
        }

        // Keep the selector path on the traversal stack so deep descendant
        // pseudo-state rules do not rebuild their ancestor chain per node.
        let mut changed_count = 0;
        let mut path = runtime_selector_path(tree, component_states, root_id)?;
        changed_count += self.apply_node_style(tree, root_id, &path, mark_dirty)?;

        let mut stack = vec![RuntimeStyleFrame {
            node_id: root_id,
            next_child: 0,
        }];
        while let Some(frame) = stack.last_mut() {
            let children = tree
                .nodes
                .get(&frame.node_id)
                .ok_or(UiTreeError::MissingNode(frame.node_id))?
                .children
                .clone();
            if frame.next_child < children.len() {
                let child_id = children[frame.next_child];
                frame.next_child += 1;
                let child = tree
                    .nodes
                    .get(&child_id)
                    .ok_or(UiTreeError::MissingNode(child_id))?;
                path.push(SelectorPathNode::from_tree_node(
                    child,
                    component_states.get(child_id),
                    false,
                ));
                changed_count += self.apply_node_style(tree, child_id, &path, mark_dirty)?;
                stack.push(RuntimeStyleFrame {
                    node_id: child_id,
                    next_child: 0,
                });
                continue;
            }

            let _ = stack.pop();
            let _ = path.pop();
        }
        Ok(changed_count)
    }

    fn apply_node_style(
        &self,
        tree: &mut UiTree,
        node_id: UiNodeId,
        path: &[SelectorPathNode],
        mark_dirty: bool,
    ) -> Result<usize, UiTreeError> {
        let Some(base_attributes) = self.base_attributes.get(&node_id) else {
            return Ok(0);
        };
        let mut node_style = UiV2ResolvedStyle::default();
        for rule in &self.rules {
            if rule.selector.matches_path(path) {
                node_style.merge_block(&rule.set);
            }
        }

        let mut next_attributes = base_attributes.clone();
        next_attributes.extend(node_style.self_values.clone());
        if let Some(current) = path.last() {
            apply_retained_runtime_state_attributes(&mut next_attributes, &current.states);
        }
        let mut next_style_overrides = self
            .base_style_overrides
            .get(&node_id)
            .cloned()
            .unwrap_or_default();
        next_style_overrides.extend(node_style.self_values);

        let node = tree
            .nodes
            .get_mut(&node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_mut() else {
            return Ok(0);
        };
        if metadata.attributes == next_attributes
            && metadata.style_overrides == next_style_overrides
        {
            return Ok(0);
        }

        let dirty = dirty_for_runtime_style_delta(&metadata.attributes, &next_attributes);
        metadata.attributes = next_attributes;
        metadata.style_overrides = next_style_overrides;
        if mark_dirty {
            merge_dirty_flags_into(&mut node.dirty, dirty);
        }
        Ok(1)
    }
}

fn collect_rules(document: &UiV2AssetDocument) -> Result<Vec<ResolvedRule>, UiV2AssetError> {
    let mut rules = Vec::new();
    let mut order = 0usize;
    for stylesheet in &document.stylesheets {
        for rule in &stylesheet.rules {
            let selector =
                UiSelector::parse(&rule.selector).map_err(|_| UiV2AssetError::InvalidSelector {
                    asset_id: document.asset.id.clone(),
                    selector: rule.selector.clone(),
                })?;
            rules.push(ResolvedRule {
                specificity: selector.specificity(),
                order,
                selector,
                set: rule.set.clone(),
            });
            order += 1;
        }
    }
    rules.sort_by_key(|rule| (rule.specificity, rule.order));
    Ok(rules)
}

fn resolve_value_map(
    values: &mut BTreeMap<String, Value>,
    tokens: &BTreeMap<String, Value>,
    depth: usize,
) {
    for value in values.values_mut() {
        resolve_value(value, tokens, depth);
    }
}

fn resolve_value(value: &mut Value, tokens: &BTreeMap<String, Value>, depth: usize) {
    if depth >= 8 {
        return;
    }
    match value {
        Value::String(raw) => {
            if let Some(replacement) = token_name(raw).and_then(|token| tokens.get(token).cloned())
            {
                *value = replacement;
                resolve_value(value, tokens, depth + 1);
            }
        }
        Value::Array(values) => {
            for value in values {
                resolve_value(value, tokens, depth + 1);
            }
        }
        Value::Table(table) => {
            for (_, value) in table.iter_mut() {
                resolve_value(value, tokens, depth + 1);
            }
        }
        _ => {}
    }
}

fn token_name(value: &str) -> Option<&str> {
    value
        .strip_prefix('$')
        .filter(|token| !token.is_empty())
        .or_else(|| {
            value
                .strip_prefix("var(")
                .and_then(|value| value.strip_suffix(')'))
        })
}

#[derive(Clone, Debug, PartialEq)]
struct ResolvedRule {
    selector: UiSelector,
    specificity: usize,
    order: usize,
    set: UiV2StyleDeclarationBlock,
}

impl ResolvedRule {
    fn uses_pseudo_state(&self) -> bool {
        selector_uses_pseudo_state(&self.selector)
    }
}

fn selector_uses_pseudo_state(selector: &UiSelector) -> bool {
    selector
        .segments
        .iter()
        .flat_map(|segment| segment.tokens.iter())
        .any(|token| matches!(token, UiSelectorToken::State(_)))
}

struct StyleFrame {
    handle: UiV2NodeHandle,
    next_child: usize,
    entered: bool,
}

impl StyleFrame {
    const fn new(handle: UiV2NodeHandle) -> Self {
        Self {
            handle,
            next_child: 0,
            entered: false,
        }
    }
}

struct RuntimeStyleFrame {
    node_id: UiNodeId,
    next_child: usize,
}

#[derive(Clone, Debug)]
struct SelectorPathNode {
    component: String,
    control_id: Option<String>,
    classes: Vec<String>,
    states: Vec<String>,
    is_host: bool,
}

impl SelectorPathNode {
    fn from_arena_node(
        node: &zircon_runtime_interface::ui::v2::UiV2ArenaNode,
        is_host: bool,
    ) -> Self {
        Self {
            component: node.component.clone(),
            control_id: node.control_id.clone(),
            classes: node.classes.clone(),
            states: collect_pseudo_states(node),
            is_host,
        }
    }

    fn from_tree_node(
        node: &UiTreeNode,
        component_state: Option<&UiComponentState>,
        is_host: bool,
    ) -> Self {
        let metadata = node.template_metadata.as_ref();
        Self {
            component: metadata
                .map(|metadata| metadata.component.clone())
                .unwrap_or_default(),
            control_id: metadata.and_then(|metadata| metadata.control_id.clone()),
            classes: metadata
                .map(|metadata| metadata.classes.clone())
                .unwrap_or_default(),
            states: collect_runtime_pseudo_states(node, component_state),
            is_host,
        }
    }
}

fn runtime_selector_path(
    tree: &UiTree,
    component_states: &crate::ui::surface::UiSurfaceComponentStateStore,
    node_id: UiNodeId,
) -> Result<Vec<SelectorPathNode>, UiTreeError> {
    let mut ids = Vec::new();
    let mut current = Some(node_id);
    while let Some(current_id) = current {
        let node = tree
            .nodes
            .get(&current_id)
            .ok_or(UiTreeError::MissingNode(current_id))?;
        ids.push(current_id);
        current = node.parent;
    }
    ids.reverse();

    let mut path = Vec::with_capacity(ids.len());
    for (index, id) in ids.into_iter().enumerate() {
        let node = tree.nodes.get(&id).ok_or(UiTreeError::MissingNode(id))?;
        path.push(SelectorPathNode::from_tree_node(
            node,
            component_states.get(id),
            index == 0,
        ));
    }
    Ok(path)
}

fn collect_pseudo_states(node: &zircon_runtime_interface::ui::v2::UiV2ArenaNode) -> Vec<String> {
    let mut states = Vec::new();
    collect_true_state_names(&node.props, &mut states);
    collect_true_state_names(&node.state, &mut states);
    states.sort();
    states.dedup();
    states
}

fn collect_runtime_pseudo_states(
    node: &UiTreeNode,
    component_state: Option<&UiComponentState>,
) -> Vec<String> {
    let mut states = Vec::new();
    if let Some(metadata) = node.template_metadata.as_ref() {
        collect_true_runtime_state_names(&metadata.attributes, &mut states);
    }
    if let Some(component_state) = component_state {
        collect_bool_state("hovered", component_state.flags.hovered, &mut states);
        collect_bool_state("focused", component_state.flags.focused, &mut states);
        collect_bool_state("pressed", component_state.flags.pressed, &mut states);
        collect_bool_state("checked", component_state.flags.checked, &mut states);
        collect_bool_state("disabled", component_state.flags.disabled, &mut states);
        collect_bool_state("expanded", component_state.flags.expanded, &mut states);
        collect_bool_state("popup_open", component_state.flags.popup_open, &mut states);
        collect_bool_state("selected", component_state.flags.selected, &mut states);
    }
    collect_bool_state("pressed", node.state_flags.pressed, &mut states);
    collect_bool_state("checked", node.state_flags.checked, &mut states);
    collect_bool_state("disabled", !node.state_flags.enabled, &mut states);
    states.sort();
    states.dedup();
    states
}

fn collect_true_state_names(values: &BTreeMap<String, Value>, states: &mut Vec<String>) {
    for (name, value) in values {
        if value.as_bool() != Some(true) {
            continue;
        }
        push_state_with_alias(name, states);
    }
}

fn collect_true_runtime_state_names(values: &BTreeMap<String, Value>, states: &mut Vec<String>) {
    for (name, value) in values {
        if value.as_bool() == Some(true) && !is_retained_runtime_state(name) {
            push_state_with_alias(name, states);
        }
    }
}

fn collect_bool_state(name: &str, enabled: bool, states: &mut Vec<String>) {
    if enabled {
        push_state_with_alias(name, states);
    }
}

fn push_state_with_alias(name: &str, states: &mut Vec<String>) {
    if !states.iter().any(|state| state == name) {
        states.push(name.to_string());
    }
    if let Some(alias) = pseudo_alias(name) {
        if !states.iter().any(|state| state == alias) {
            states.push(alias.to_string());
        }
    }
}

fn is_retained_runtime_state(name: &str) -> bool {
    matches!(
        name,
        "hover"
            | "hovered"
            | "focus"
            | "focused"
            | "active"
            | "pressed"
            | "checked"
            | "disabled"
            | "enabled"
            | "expanded"
            | "popup_open"
            | "open"
            | "selected"
    )
}

fn pseudo_alias(name: &str) -> Option<&'static str> {
    match name {
        "hovered" => Some("hover"),
        "pressed" => Some("active"),
        "focused" => Some("focus"),
        "disabled" => Some("disabled"),
        "checked" => Some("checked"),
        "selected" => Some("selected"),
        "popup_open" => Some("open"),
        _ => None,
    }
}

fn apply_retained_runtime_state_attributes(
    attributes: &mut BTreeMap<String, Value>,
    active_states: &[String],
) {
    let retained_keys = [
        "hover",
        "hovered",
        "focus",
        "focused",
        "active",
        "pressed",
        "checked",
        "disabled",
        "enabled",
        "expanded",
        "popup_open",
        "open",
        "selected",
    ];
    for key in retained_keys {
        attributes.remove(key);
    }
    for state in [
        "hovered",
        "focused",
        "pressed",
        "checked",
        "disabled",
        "expanded",
        "popup_open",
        "selected",
    ] {
        if active_states.iter().any(|active| active == state) {
            attributes.insert(state.to_string(), Value::Boolean(true));
        }
    }
}

trait UiV2SelectorMatchExt {
    fn matches_path(&self, path: &[SelectorPathNode]) -> bool;
}

impl UiV2SelectorMatchExt for UiSelector {
    fn matches_path(&self, path: &[SelectorPathNode]) -> bool {
        if path.is_empty() || self.segments.is_empty() {
            return false;
        }

        let mut path_index = path.len() - 1;
        let mut selector_index = self.segments.len() - 1;
        if !matches_segment(&self.segments[selector_index], &path[path_index]) {
            return false;
        }

        while selector_index > 0 {
            let combinator = self.segments[selector_index].combinator;
            selector_index -= 1;
            match combinator {
                Some(UiSelectorCombinator::Child) => {
                    if path_index == 0 {
                        return false;
                    }
                    path_index -= 1;
                    if !matches_segment(&self.segments[selector_index], &path[path_index]) {
                        return false;
                    }
                }
                Some(UiSelectorCombinator::Descendant) => {
                    let mut matched = None;
                    let mut candidate = path_index;
                    while candidate > 0 {
                        candidate -= 1;
                        if matches_segment(&self.segments[selector_index], &path[candidate]) {
                            matched = Some(candidate);
                            break;
                        }
                    }
                    let Some(found) = matched else {
                        return false;
                    };
                    path_index = found;
                }
                None => return false,
            }
        }

        true
    }
}

fn matches_segment(segment: &UiSelectorSegment, node: &SelectorPathNode) -> bool {
    segment.tokens.iter().all(|token| match token {
        UiSelectorToken::Type(component) => node.component == *component,
        UiSelectorToken::Class(class_name) => node.classes.iter().any(|class| class == class_name),
        UiSelectorToken::Id(control_id) => node.control_id.as_ref() == Some(control_id),
        UiSelectorToken::State(state) => node.states.iter().any(|value| value == state),
        UiSelectorToken::Part(_) => false,
        UiSelectorToken::Host => node.is_host,
    })
}

fn dirty_for_runtime_style_delta(
    old_attributes: &BTreeMap<String, Value>,
    new_attributes: &BTreeMap<String, Value>,
) -> UiDirtyFlags {
    let mut dirty = UiDirtyFlags {
        render: true,
        ..UiDirtyFlags::default()
    };
    let changed_keys = old_attributes
        .keys()
        .chain(new_attributes.keys())
        .filter(|key| old_attributes.get(*key) != new_attributes.get(*key))
        .cloned()
        .collect::<BTreeSet<_>>();
    for key in changed_keys {
        if is_retained_runtime_state(&key) {
            continue;
        }
        if is_text_affecting_style_key(&key) {
            dirty.text = true;
        } else if !is_render_only_style_key(&key) {
            dirty.style = true;
        }
    }
    dirty
}

fn is_text_affecting_style_key(key: &str) -> bool {
    matches!(
        key,
        "text"
            | "label"
            | "font"
            | "font_size"
            | "font_family"
            | "font_weight"
            | "line_height"
            | "letter_spacing"
            | "text_align"
            | "wrap"
    )
}

fn is_render_only_style_key(key: &str) -> bool {
    matches!(
        key,
        "background"
            | "background_color"
            | "fg"
            | "foreground"
            | "foreground_color"
            | "color"
            | "border"
            | "border_color"
            | "border_width"
            | "outline"
            | "outline_color"
            | "outline_width"
            | "opacity"
            | "radius"
            | "corner_radius"
            | "shadow"
            | "elevation"
            | "cursor"
    )
}

fn merge_dirty_flags_into(target: &mut UiDirtyFlags, dirty: UiDirtyFlags) {
    target.layout |= dirty.layout;
    target.hit_test |= dirty.hit_test;
    target.render |= dirty.render;
    target.style |= dirty.style;
    target.text |= dirty.text;
    target.input |= dirty.input;
    target.visible_range |= dirty.visible_range;
}
