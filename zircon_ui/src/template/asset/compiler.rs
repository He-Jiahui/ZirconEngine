use std::collections::{BTreeMap, BTreeSet};

use toml::{map::Map, Value};

use crate::{
    UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetKind, UiChildMount, UiComponentDefinition,
    UiNodeDefinition, UiNodeDefinitionKind, UiSelector, UiStyleDeclarationBlock, UiStyleSheet,
    UiTemplateInstance, UiTemplateNode,
};

use super::style::UiSelectorMatchNode;

#[derive(Clone, Debug, PartialEq)]
pub struct UiCompiledDocument {
    pub asset: UiAssetHeader,
    instance: UiTemplateInstance,
}

impl UiCompiledDocument {
    pub fn into_template_instance(self) -> UiTemplateInstance {
        self.instance
    }

    pub fn template_instance(&self) -> &UiTemplateInstance {
        &self.instance
    }
}

#[derive(Default)]
pub struct UiDocumentCompiler {
    widget_imports: BTreeMap<String, UiAssetDocument>,
    style_imports: BTreeMap<String, UiAssetDocument>,
}

impl UiDocumentCompiler {
    pub fn register_widget_import(
        &mut self,
        reference: impl Into<String>,
        document: UiAssetDocument,
    ) -> Result<&mut Self, UiAssetError> {
        let reference = reference.into();
        if document.asset.kind != UiAssetKind::Widget {
            return Err(UiAssetError::ImportKindMismatch {
                reference,
                expected: UiAssetKind::Widget,
                actual: document.asset.kind,
            });
        }
        let _ = self.widget_imports.insert(reference, document);
        Ok(self)
    }

    pub fn register_style_import(
        &mut self,
        reference: impl Into<String>,
        document: UiAssetDocument,
    ) -> Result<&mut Self, UiAssetError> {
        let reference = reference.into();
        if document.asset.kind != UiAssetKind::Style {
            return Err(UiAssetError::ImportKindMismatch {
                reference,
                expected: UiAssetKind::Style,
                actual: document.asset.kind,
            });
        }
        let _ = self.style_imports.insert(reference, document);
        Ok(self)
    }

    pub fn compile(&self, document: &UiAssetDocument) -> Result<UiCompiledDocument, UiAssetError> {
        validate_document_shape(document)?;
        let root_id = document
            .root
            .as_ref()
            .ok_or_else(|| UiAssetError::InvalidDocument {
                asset_id: document.asset.id.clone(),
                detail: "layout/widget assets require a root node".to_string(),
            })?
            .node
            .clone();

        let mut artifacts = CompilationArtifacts::default();
        let tokens = compose_tokens(&BTreeMap::new(), &document.tokens);
        let mut roots = self.expand_node(
            document,
            &root_id,
            &tokens,
            &BTreeMap::new(),
            None,
            &mut artifacts,
        )?;
        let root = roots
            .drain(..)
            .next()
            .ok_or_else(|| UiAssetError::InvalidDocument {
                asset_id: document.asset.id.clone(),
                detail: "asset expansion produced no root nodes".to_string(),
            })?;

        let mut instance = UiTemplateInstance { root };
        UiStyleResolver::apply(document, self, &mut instance.root, &artifacts)?;

        Ok(UiCompiledDocument {
            asset: document.asset.clone(),
            instance,
        })
    }

    fn expand_node(
        &self,
        document: &UiAssetDocument,
        node_id: &str,
        tokens: &BTreeMap<String, Value>,
        params: &BTreeMap<String, Value>,
        slot_fills: Option<&BTreeMap<String, Vec<UiTemplateNode>>>,
        artifacts: &mut CompilationArtifacts,
    ) -> Result<Vec<UiTemplateNode>, UiAssetError> {
        let node = document
            .nodes
            .get(node_id)
            .ok_or_else(|| UiAssetError::MissingNode {
                asset_id: document.asset.id.clone(),
                node_id: node_id.to_string(),
            })?;

        match node.kind {
            UiNodeDefinitionKind::Native => {
                self.expand_native_node(document, node, tokens, params, slot_fills, artifacts)
            }
            UiNodeDefinitionKind::Component => {
                let component_name =
                    node.component
                        .as_deref()
                        .ok_or_else(|| UiAssetError::InvalidDocument {
                            asset_id: document.asset.id.clone(),
                            detail: format!("component node {node_id} missing component name"),
                        })?;
                self.expand_component_instance(
                    document,
                    component_name,
                    node,
                    tokens,
                    document,
                    tokens,
                    params,
                    artifacts,
                )
            }
            UiNodeDefinitionKind::Reference => {
                let reference =
                    node.component_ref
                        .as_deref()
                        .ok_or_else(|| UiAssetError::InvalidDocument {
                            asset_id: document.asset.id.clone(),
                            detail: format!("reference node {node_id} missing component_ref"),
                        })?;
                let imported = self.widget_imports.get(reference).ok_or_else(|| {
                    UiAssetError::UnknownImport {
                        reference: reference.to_string(),
                    }
                })?;
                let component_name = component_name_from_reference(reference)?;
                self.expand_component_instance(
                    imported,
                    &component_name,
                    node,
                    &compose_tokens(tokens, &imported.tokens),
                    document,
                    tokens,
                    params,
                    artifacts,
                )
            }
            UiNodeDefinitionKind::Slot => {
                let slot_name =
                    node.slot_name
                        .as_deref()
                        .ok_or_else(|| UiAssetError::InvalidDocument {
                            asset_id: document.asset.id.clone(),
                            detail: format!("slot node {node_id} missing slot_name"),
                        })?;
                Ok(slot_fills
                    .and_then(|fills| fills.get(slot_name))
                    .cloned()
                    .unwrap_or_default())
            }
        }
    }

    fn expand_native_node(
        &self,
        document: &UiAssetDocument,
        node: &UiNodeDefinition,
        tokens: &BTreeMap<String, Value>,
        params: &BTreeMap<String, Value>,
        slot_fills: Option<&BTreeMap<String, Vec<UiTemplateNode>>>,
        artifacts: &mut CompilationArtifacts,
    ) -> Result<Vec<UiTemplateNode>, UiAssetError> {
        let component = node
            .widget_type
            .as_ref()
            .ok_or_else(|| UiAssetError::InvalidDocument {
                asset_id: document.asset.id.clone(),
                detail: "native node missing type".to_string(),
            })?
            .clone();
        let mut children = Vec::new();
        for child in &node.children {
            let expanded = self.expand_node(
                document,
                &child.child,
                tokens,
                params,
                slot_fills,
                artifacts,
            )?;
            children.extend(apply_child_mount(expanded, child, tokens, params));
        }

        Ok(vec![UiTemplateNode {
            component: Some(component),
            template: None,
            slot: None,
            control_id: node.control_id.clone(),
            classes: node.classes.clone(),
            bindings: node.bindings.clone(),
            children,
            slots: BTreeMap::new(),
            attributes: build_attribute_map(node, tokens, params),
            slot_attributes: BTreeMap::new(),
            style_overrides: resolve_value_map(&node.style_overrides.self_values, tokens, params),
            style_tokens: BTreeMap::new(),
        }])
    }

    fn expand_component_instance(
        &self,
        document: &UiAssetDocument,
        component_name: &str,
        instance_node: &UiNodeDefinition,
        tokens: &BTreeMap<String, Value>,
        caller_document: &UiAssetDocument,
        caller_tokens: &BTreeMap<String, Value>,
        params: &BTreeMap<String, Value>,
        artifacts: &mut CompilationArtifacts,
    ) -> Result<Vec<UiTemplateNode>, UiAssetError> {
        let component = document.components.get(component_name).ok_or_else(|| {
            UiAssetError::UnknownComponent {
                asset_id: document.asset.id.clone(),
                component: component_name.to_string(),
            }
        })?;
        if document.asset.kind == UiAssetKind::Widget {
            artifacts.record_widget_styles(document, tokens);
        }

        validate_slot_mounts(component_name, component, &instance_node.children)?;

        let component_params =
            resolve_component_params(component, &instance_node.params, tokens, params);
        let mut fills = BTreeMap::new();
        for child in &instance_node.children {
            let mount_name = child.mount.clone().unwrap_or_default();
            let expanded = self.expand_node(
                caller_document,
                &child.child,
                caller_tokens,
                params,
                None,
                artifacts,
            )?;
            fills
                .entry(mount_name)
                .or_insert_with(Vec::new)
                .extend(apply_child_mount(expanded, child, tokens, params));
        }

        let component_tokens = compose_tokens(tokens, &document.tokens);
        let mut roots = self.expand_node(
            document,
            &component.root,
            &component_tokens,
            &component_params,
            Some(&fills),
            artifacts,
        )?;
        if roots.len() != 1 {
            return Err(UiAssetError::InvalidDocument {
                asset_id: document.asset.id.clone(),
                detail: format!("component {component_name} must expand to exactly one root node"),
            });
        }

        let mut root = roots.remove(0);
        decorate_component_root(&mut root, instance_node, tokens, params);
        Ok(vec![root])
    }
}

#[derive(Default)]
struct CompilationArtifacts {
    widget_styles: Vec<ResolvedStyleSheet>,
    seen_widget_assets: BTreeSet<String>,
}

impl CompilationArtifacts {
    fn record_widget_styles(
        &mut self,
        document: &UiAssetDocument,
        inherited: &BTreeMap<String, Value>,
    ) {
        if !self.seen_widget_assets.insert(document.asset.id.clone()) {
            return;
        }
        let tokens = compose_tokens(inherited, &document.tokens);
        for stylesheet in &document.stylesheets {
            self.widget_styles.push(ResolvedStyleSheet {
                stylesheet: stylesheet.clone(),
                tokens: tokens.clone(),
            });
        }
    }
}

#[derive(Clone)]
struct ResolvedStyleSheet {
    stylesheet: UiStyleSheet,
    tokens: BTreeMap<String, Value>,
}

#[derive(Default)]
pub struct UiStyleResolver;

impl UiStyleResolver {
    fn apply(
        document: &UiAssetDocument,
        compiler: &UiDocumentCompiler,
        root: &mut UiTemplateNode,
        artifacts: &CompilationArtifacts,
    ) -> Result<(), UiAssetError> {
        let mut sheets = artifacts.widget_styles.clone();
        for reference in &document.imports.styles {
            let imported = compiler.style_imports.get(reference).ok_or_else(|| {
                UiAssetError::UnknownImport {
                    reference: reference.clone(),
                }
            })?;
            let tokens = compose_tokens(&document.tokens, &imported.tokens);
            for stylesheet in &imported.stylesheets {
                sheets.push(ResolvedStyleSheet {
                    stylesheet: stylesheet.clone(),
                    tokens: tokens.clone(),
                });
            }
        }
        for stylesheet in &document.stylesheets {
            sheets.push(ResolvedStyleSheet {
                stylesheet: stylesheet.clone(),
                tokens: document.tokens.clone(),
            });
        }

        let parsed = build_style_plan(&sheets)?;
        let mut path = Vec::new();
        apply_styles_to_tree(root, &parsed, &mut path);
        Ok(())
    }
}

#[derive(Clone)]
struct ParsedStyleRule {
    selector: UiSelector,
    specificity: usize,
    order: usize,
    set: UiStyleDeclarationBlock,
    tokens: BTreeMap<String, Value>,
}

fn build_style_plan(sheets: &[ResolvedStyleSheet]) -> Result<Vec<ParsedStyleRule>, UiAssetError> {
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

fn apply_styles_to_tree(
    node: &mut UiTemplateNode,
    rules: &[ParsedStyleRule],
    path: &mut Vec<StylePathEntry>,
) {
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

    if !node.style_overrides.is_empty() {
        let inline = node.style_overrides.clone();
        merge_value_maps(&mut node.attributes, &inline);
    }

    for child in &mut node.children {
        apply_styles_to_tree(child, rules, path);
    }

    let _ = path.pop();
}

#[derive(Clone)]
struct StylePathEntry {
    component: String,
    control_id: Option<String>,
    classes: Vec<String>,
    is_host: bool,
}

impl StylePathEntry {
    fn from_node(node: &UiTemplateNode, is_host: bool) -> Self {
        Self {
            component: node.component.clone().unwrap_or_default(),
            control_id: node.control_id.clone(),
            classes: node.classes.clone(),
            is_host,
        }
    }

    fn as_match_node(&self) -> UiSelectorMatchNode<'_> {
        UiSelectorMatchNode {
            component: &self.component,
            control_id: self.control_id.as_deref(),
            classes: &self.classes,
            is_host: self.is_host,
            states: &[],
        }
    }
}

fn validate_document_shape(document: &UiAssetDocument) -> Result<(), UiAssetError> {
    match document.asset.kind {
        UiAssetKind::Layout | UiAssetKind::Widget => {
            if document.root.is_none() {
                return Err(UiAssetError::InvalidDocument {
                    asset_id: document.asset.id.clone(),
                    detail: "layout/widget assets require [root]".to_string(),
                });
            }
        }
        UiAssetKind::Style => {}
    }
    Ok(())
}

fn build_attribute_map(
    node: &UiNodeDefinition,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    let mut attributes = resolve_value_map(&node.props, tokens, params);
    if let Some(layout) = &node.layout {
        let mut layout = resolve_value_map(layout, tokens, params);
        normalize_layout_table(&mut layout);
        let _ = attributes.insert("layout".to_string(), table_value(layout));
    }
    attributes
}

fn apply_child_mount(
    nodes: Vec<UiTemplateNode>,
    child: &UiChildMount,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) -> Vec<UiTemplateNode> {
    let mut slot = resolve_value_map(&child.slot, tokens, params);
    normalize_layout(&mut slot);
    nodes
        .into_iter()
        .map(|mut node| {
            merge_value_maps(&mut node.slot_attributes, &slot);
            node
        })
        .collect()
}

fn resolve_component_params(
    component: &UiComponentDefinition,
    provided: &BTreeMap<String, Value>,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    let mut resolved = BTreeMap::new();
    for (name, schema) in &component.params {
        if let Some(default) = &schema.default {
            let _ = resolved.insert(name.clone(), resolve_value(default, tokens, params));
        }
    }
    for (name, value) in provided {
        let _ = resolved.insert(name.clone(), resolve_value(value, tokens, params));
    }
    resolved
}

fn decorate_component_root(
    root: &mut UiTemplateNode,
    instance_node: &UiNodeDefinition,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) {
    if let Some(control_id) = &instance_node.control_id {
        root.control_id = Some(control_id.clone());
    }
    append_classes(&mut root.classes, &instance_node.classes);
    let inline = resolve_value_map(&instance_node.style_overrides.self_values, tokens, params);
    merge_value_maps(&mut root.style_overrides, &inline);
    merge_value_maps_resolved(
        &mut root.slot_attributes,
        &instance_node.style_overrides.slot,
        tokens,
        params,
    );
}

fn validate_slot_mounts(
    component_name: &str,
    component: &UiComponentDefinition,
    children: &[UiChildMount],
) -> Result<(), UiAssetError> {
    let mut counts = BTreeMap::<String, usize>::new();
    for child in children {
        let slot_name = child.mount.clone().unwrap_or_default();
        let slot = component
            .slots
            .get(&slot_name)
            .ok_or_else(|| UiAssetError::UnknownSlot {
                component: component_name.to_string(),
                slot_name: slot_name.clone(),
            })?;
        let count = counts.entry(slot_name.clone()).or_insert(0);
        *count += 1;
        if !slot.multiple && *count > 1 {
            return Err(UiAssetError::SlotDoesNotAcceptMultiple {
                component: component_name.to_string(),
                slot_name,
            });
        }
    }

    for (slot_name, slot) in &component.slots {
        if slot.required && !counts.contains_key(slot_name) {
            return Err(UiAssetError::MissingRequiredSlot {
                component: component_name.to_string(),
                slot_name: slot_name.clone(),
            });
        }
    }

    Ok(())
}

fn component_name_from_reference(reference: &str) -> Result<String, UiAssetError> {
    reference
        .split_once('#')
        .map(|(_, component)| component.to_string())
        .ok_or_else(|| UiAssetError::InvalidDocument {
            asset_id: reference.to_string(),
            detail: "component references must include a #Component suffix".to_string(),
        })
}

fn compose_tokens(
    inherited: &BTreeMap<String, Value>,
    local: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    let mut tokens = inherited.clone();
    merge_value_maps(&mut tokens, local);
    tokens
}

fn resolve_value_map(
    values: &BTreeMap<String, Value>,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    values
        .iter()
        .map(|(key, value)| (key.clone(), resolve_value(value, tokens, params)))
        .collect()
}

fn resolve_value(
    value: &Value,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) -> Value {
    match value {
        Value::String(value) => {
            if let Some(param_name) = value.strip_prefix("$param.") {
                params
                    .get(param_name)
                    .cloned()
                    .unwrap_or_else(|| Value::String(value.clone()))
            } else if let Some(token_name) = value.strip_prefix('$') {
                tokens
                    .get(token_name)
                    .cloned()
                    .unwrap_or_else(|| Value::String(value.clone()))
            } else {
                Value::String(value.clone())
            }
        }
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| resolve_value(value, tokens, params))
                .collect(),
        ),
        Value::Table(values) => Value::Table(
            values
                .iter()
                .map(|(key, value)| (key.clone(), resolve_value(value, tokens, params)))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn merge_value_maps_resolved(
    target: &mut BTreeMap<String, Value>,
    overlay: &BTreeMap<String, Value>,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) {
    let resolved = resolve_value_map(overlay, tokens, params);
    merge_value_maps(target, &resolved);
}

fn merge_value_maps(target: &mut BTreeMap<String, Value>, overlay: &BTreeMap<String, Value>) {
    for (key, value) in overlay {
        if let Some(current) = target.get_mut(key) {
            merge_value(current, value);
        } else {
            let _ = target.insert(key.clone(), value.clone());
        }
    }
    normalize_layout(target);
}

fn merge_value(current: &mut Value, overlay: &Value) {
    match (current, overlay) {
        (Value::Table(current_table), Value::Table(overlay_table)) => {
            for (key, value) in overlay_table {
                if let Some(existing) = current_table.get_mut(key) {
                    merge_value(existing, value);
                } else {
                    let _ = current_table.insert(key.clone(), value.clone());
                }
            }
        }
        (current, overlay) => *current = overlay.clone(),
    }
}

fn normalize_layout(values: &mut BTreeMap<String, Value>) {
    let Some(Value::Table(layout)) = values.get_mut("layout") else {
        return;
    };
    normalize_layout_table_map(layout);
}

fn normalize_layout_table(values: &mut BTreeMap<String, Value>) {
    normalize_axis_entry(values.get_mut("width"));
    normalize_axis_entry(values.get_mut("height"));
}

fn normalize_layout_table_map(values: &mut Map<String, Value>) {
    normalize_axis_entry(values.get_mut("width"));
    normalize_axis_entry(values.get_mut("height"));
}

fn normalize_axis_entry(value: Option<&mut Value>) {
    normalize_axis_table(value);
}

fn normalize_axis_table(value: Option<&mut Value>) {
    let Some(Value::Table(table)) = value else {
        return;
    };
    if table.get("stretch").and_then(Value::as_str) == Some("Fixed") {
        if let Some(preferred) = table.get("preferred").cloned() {
            let _ = table.insert("min".to_string(), preferred.clone());
            let _ = table.insert("max".to_string(), preferred);
        }
    }
}

fn table_value(values: BTreeMap<String, Value>) -> Value {
    Value::Table(values.into_iter().collect::<Map<String, Value>>())
}

fn append_classes(target: &mut Vec<String>, extra: &[String]) {
    for class_name in extra {
        if !target.iter().any(|value| value == class_name) {
            target.push(class_name.clone());
        }
    }
}
