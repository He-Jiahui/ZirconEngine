use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::component::UiComponentState;
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};
use zircon_runtime_interface::ui::template::{
    UiSelector, UiSelectorCombinator, UiSelectorSegment, UiSelectorToken,
};
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError, UiTreeNode};
use zircon_runtime_interface::ui::v2::{
    UiV2AssetDocument, UiV2AssetError, UiV2NodeArena, UiV2NodeHandle, UiV2ResolvedStyle,
    UiV2ResolvedStyleSheet, UiV2StyleDeclarationBlock,
};

use runtime_state::{
    apply_retained_runtime_state_attributes, collect_pseudo_states, collect_runtime_pseudo_states,
    dirty_for_runtime_style_delta, merge_dirty_flags_into,
};

mod runtime_state;

#[derive(Default)]
pub struct UiV2StyleResolver;

impl UiV2StyleResolver {
    pub fn resolve(
        document: &UiV2AssetDocument,
        arena: &UiV2NodeArena,
    ) -> Result<UiV2ResolvedStyleSheet, UiV2AssetError> {
        let rules = collect_rules(document)?;
        Self::resolve_with_rules(document, arena, &rules, true, None)
    }

    pub fn resolve_with_theme(
        document: &UiV2AssetDocument,
        arena: &UiV2NodeArena,
        theme: &crate::ui::theme::UiThemeRegistry,
    ) -> Result<UiV2ResolvedStyleSheet, UiV2AssetError> {
        let rules = collect_rules(document)?;
        Self::resolve_with_rules(document, arena, &rules, true, Some(theme))
    }

    pub(crate) fn resolve_static(
        document: &UiV2AssetDocument,
        arena: &UiV2NodeArena,
    ) -> Result<UiV2ResolvedStyleSheet, UiV2AssetError> {
        let rules = collect_rules(document)?
            .into_iter()
            .filter(|rule| !rule.uses_pseudo_state())
            .collect::<Vec<_>>();
        Self::resolve_with_rules(document, arena, &rules, false, None)
    }

    pub(crate) fn resolve_static_with_theme(
        document: &UiV2AssetDocument,
        arena: &UiV2NodeArena,
        theme: &crate::ui::theme::UiThemeRegistry,
    ) -> Result<UiV2ResolvedStyleSheet, UiV2AssetError> {
        let rules = collect_rules(document)?
            .into_iter()
            .filter(|rule| !rule.uses_pseudo_state())
            .collect::<Vec<_>>();
        Self::resolve_with_rules(document, arena, &rules, false, Some(theme))
    }

    fn resolve_with_rules(
        document: &UiV2AssetDocument,
        arena: &UiV2NodeArena,
        rules: &[ResolvedRule],
        include_inline_style: bool,
        theme: Option<&crate::ui::theme::UiThemeRegistry>,
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
                        merge_block_with_token_sources(&mut node_style, &rule.set, document);
                    }
                }
                if include_inline_style {
                    merge_block_with_token_sources(&mut node_style, &node.style, document);
                }
                resolve_value_map(&mut node_style.self_values, &document.tokens, theme, 0);
                resolve_value_map(&mut node_style.slot, &document.tokens, theme, 0);
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
    base_style_tokens: BTreeMap<UiNodeId, BTreeMap<String, String>>,
}

impl UiV2RuntimeStyleIndex {
    pub(crate) fn from_document(document: &UiV2AssetDocument) -> Result<Self, UiV2AssetError> {
        Self::from_document_with_optional_theme(document, None)
    }

    pub(crate) fn from_document_with_theme(
        document: &UiV2AssetDocument,
        theme: &crate::ui::theme::UiThemeRegistry,
    ) -> Result<Self, UiV2AssetError> {
        Self::from_document_with_optional_theme(document, Some(theme))
    }

    fn from_document_with_optional_theme(
        document: &UiV2AssetDocument,
        theme: Option<&crate::ui::theme::UiThemeRegistry>,
    ) -> Result<Self, UiV2AssetError> {
        let mut rules = collect_rules(document)?
            .into_iter()
            .filter(ResolvedRule::uses_pseudo_state)
            .collect::<Vec<_>>();
        for rule in &mut rules {
            rule.style_tokens = style_token_sources_for_block(&rule.set, document);
            resolve_value_map(&mut rule.set.self_values, &document.tokens, theme, 0);
            resolve_value_map(&mut rule.set.slot, &document.tokens, theme, 0);
        }
        Ok(Self {
            rules,
            base_attributes: BTreeMap::new(),
            base_style_overrides: BTreeMap::new(),
            base_style_tokens: BTreeMap::new(),
        })
    }

    pub(crate) fn has_runtime_rules(&self) -> bool {
        !self.rules.is_empty()
    }

    pub(crate) fn capture_baseline_from_tree(&mut self, tree: &UiTree) {
        self.base_attributes.clear();
        self.base_style_overrides.clear();
        self.base_style_tokens.clear();
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
            let _ = self
                .base_style_tokens
                .insert(*node_id, metadata.style_tokens.clone());
        }
    }

    pub(crate) fn set_base_attribute(
        &mut self,
        node_id: UiNodeId,
        property: String,
        value: Value,
    ) -> bool {
        let Some(attributes) = self.base_attributes.get_mut(&node_id) else {
            return false;
        };
        if attributes.get(&property) == Some(&value) {
            return false;
        }
        if let Some(tokens) = self.base_style_tokens.get_mut(&node_id) {
            remove_style_token_sources(tokens, &property);
        }
        attributes.insert(property, value);
        true
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
                merge_runtime_rule(&mut node_style, rule);
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
        for key in node_style.self_values.keys() {
            if self
                .base_style_overrides
                .get(&node_id)
                .and_then(|values| values.get(key))
                .is_some_and(|override_value| base_attributes.get(key) != Some(override_value))
            {
                continue;
            }
            if let Some(value) = next_attributes.get(key).cloned() {
                let _ = next_style_overrides.insert(key.clone(), value);
            }
        }
        let mut next_style_tokens = self
            .base_style_tokens
            .get(&node_id)
            .cloned()
            .unwrap_or_default();
        for key in node_style.self_values.keys() {
            remove_style_token_sources(&mut next_style_tokens, key);
        }
        for (key, source) in &node_style.style_tokens {
            let _ = next_style_tokens.insert(key.clone(), source.clone());
        }

        let node = tree
            .nodes
            .get_mut(&node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_mut() else {
            return Ok(0);
        };
        if metadata.attributes == next_attributes
            && metadata.style_overrides == next_style_overrides
            && metadata.style_tokens == next_style_tokens
        {
            return Ok(0);
        }

        let dirty = dirty_for_runtime_style_delta(&metadata.attributes, &next_attributes);
        metadata.attributes = next_attributes;
        metadata.style_overrides = next_style_overrides;
        metadata.style_tokens = next_style_tokens;
        if mark_dirty {
            merge_dirty_flags_into(&mut node.dirty, dirty);
        }
        Ok(1)
    }
}

fn merge_block_with_token_sources(
    style: &mut UiV2ResolvedStyle,
    block: &UiV2StyleDeclarationBlock,
    document: &UiV2AssetDocument,
) {
    merge_value_map_with_token_sources(
        &mut style.self_values,
        &mut style.style_tokens,
        None,
        &block.self_values,
        document,
    );
    merge_value_map_with_token_sources(
        &mut style.slot,
        &mut style.style_tokens,
        Some("slot"),
        &block.slot,
        document,
    );
}

fn style_token_sources_for_block(
    block: &UiV2StyleDeclarationBlock,
    document: &UiV2AssetDocument,
) -> BTreeMap<String, String> {
    let mut tokens = BTreeMap::new();
    collect_value_map_token_sources(None, &block.self_values, &mut tokens, document);
    collect_value_map_token_sources(Some("slot"), &block.slot, &mut tokens, document);
    tokens
}

fn merge_runtime_rule(style: &mut UiV2ResolvedStyle, rule: &ResolvedRule) {
    for (key, value) in &rule.set.self_values {
        remove_style_token_sources(&mut style.style_tokens, key);
        let _ = style.self_values.insert(key.clone(), value.clone());
    }
    for (key, value) in &rule.set.slot {
        let path = style_token_path(Some("slot"), key);
        remove_style_token_sources(&mut style.style_tokens, &path);
        let _ = style.slot.insert(key.clone(), value.clone());
    }
    for (key, source) in &rule.style_tokens {
        let _ = style.style_tokens.insert(key.clone(), source.clone());
    }
}

fn merge_value_map_with_token_sources(
    target: &mut BTreeMap<String, Value>,
    style_tokens: &mut BTreeMap<String, String>,
    prefix: Option<&str>,
    values: &BTreeMap<String, Value>,
    document: &UiV2AssetDocument,
) {
    for (key, value) in values {
        let path = style_token_path(prefix, key);
        remove_style_token_sources(style_tokens, &path);
        collect_value_token_sources(&path, value, style_tokens, document);
        let _ = target.insert(key.clone(), value.clone());
    }
}

fn collect_value_map_token_sources(
    prefix: Option<&str>,
    values: &BTreeMap<String, Value>,
    style_tokens: &mut BTreeMap<String, String>,
    document: &UiV2AssetDocument,
) {
    for (key, value) in values {
        let path = style_token_path(prefix, key);
        collect_value_token_sources(&path, value, style_tokens, document);
    }
}

fn collect_value_token_sources(
    path: &str,
    value: &Value,
    style_tokens: &mut BTreeMap<String, String>,
    document: &UiV2AssetDocument,
) {
    match value {
        Value::String(raw) => {
            if let Some(source) = resolved_token_source(raw, document, 0) {
                let _ = style_tokens.insert(path.to_string(), source);
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                collect_value_token_sources(
                    &format!("{path}[{index}]"),
                    value,
                    style_tokens,
                    document,
                );
            }
        }
        Value::Table(values) => {
            for (key, value) in values {
                collect_value_token_sources(
                    &format!("{path}.{key}"),
                    value,
                    style_tokens,
                    document,
                );
            }
        }
        _ => {}
    }
}

fn style_token_path(prefix: Option<&str>, key: &str) -> String {
    if let Some(prefix) = prefix {
        format!("{prefix}.{key}")
    } else {
        key.to_string()
    }
}

fn remove_style_token_sources(style_tokens: &mut BTreeMap<String, String>, path: &str) {
    let nested = format!("{path}.");
    let indexed = format!("{path}[");
    style_tokens
        .retain(|key, _| key != path && !key.starts_with(&nested) && !key.starts_with(&indexed));
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
                style_tokens: BTreeMap::new(),
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
    theme: Option<&crate::ui::theme::UiThemeRegistry>,
    depth: usize,
) {
    for value in values.values_mut() {
        resolve_value(value, tokens, theme, depth);
    }
}

fn resolve_value(
    value: &mut Value,
    tokens: &BTreeMap<String, Value>,
    theme: Option<&crate::ui::theme::UiThemeRegistry>,
    depth: usize,
) {
    if depth >= 8 {
        return;
    }
    match value {
        Value::String(raw) => {
            if let Some(theme_value) = theme.and_then(|theme| theme_value(raw, theme)) {
                *value = theme_value;
                return;
            }
            if let Some(replacement) = token_name(raw).and_then(|token| tokens.get(token).cloned())
            {
                *value = replacement;
                resolve_value(value, tokens, theme, depth + 1);
            }
        }
        Value::Array(values) => {
            for value in values {
                resolve_value(value, tokens, theme, depth + 1);
            }
        }
        Value::Table(table) => {
            for (_, value) in table.iter_mut() {
                resolve_value(value, tokens, theme, depth + 1);
            }
        }
        _ => {}
    }
}

fn theme_value(raw: &str, theme: &crate::ui::theme::UiThemeRegistry) -> Option<Value> {
    let role = theme_role(raw)?;
    let color = theme.resolve_role(role)?;
    style_color_value(&color)
}

fn resolved_token_source(raw: &str, document: &UiV2AssetDocument, depth: usize) -> Option<String> {
    if depth >= 8 {
        return None;
    }
    if let Some(theme_source) = theme_role(raw) {
        return Some(theme_source_name(theme_source));
    }
    let token = token_name(raw)?;
    let token_source = format!("token.{token}");
    let nested_source = document.tokens.get(token).and_then(|value| {
        value
            .as_str()
            .and_then(|raw| resolved_token_source(raw, document, depth + 1))
    });
    Some(
        nested_source
            .map(|nested| format!("{token_source} -> {nested}"))
            .unwrap_or(token_source),
    )
}

fn theme_role(raw: &str) -> Option<&str> {
    let unwrapped = raw
        .strip_prefix("var(")
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(raw);
    let role = unwrapped.strip_prefix('$').unwrap_or(unwrapped);
    (role.starts_with("theme.") || role.starts_with("theme:") || role.starts_with("palette."))
        .then_some(role)
}

fn theme_source_name(role: &str) -> String {
    let normalized = role
        .strip_prefix('$')
        .unwrap_or(role)
        .strip_prefix("theme:")
        .map(|role| format!("theme.{role}"))
        .unwrap_or_else(|| {
            if role.starts_with("theme.") {
                role.to_string()
            } else {
                format!("theme.{role}")
            }
        });
    normalized
}

fn style_color_value(color: &UiStyleColor) -> Option<Value> {
    match color {
        UiStyleColor::Rgba(color) => Some(Value::String(rgba_hex(*color))),
        UiStyleColor::Transparent => Some(Value::String("transparent".to_string())),
        UiStyleColor::Inherit => Some(Value::String("inherit".to_string())),
        UiStyleColor::Role(_) => None,
    }
}

fn rgba_hex(color: UiRgbaColor) -> String {
    let [red, green, blue, alpha] = color.to_u8();
    if alpha == 255 {
        format!("#{red:02x}{green:02x}{blue:02x}")
    } else {
        format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
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
    style_tokens: BTreeMap<String, String>,
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
