use std::collections::{BTreeMap, BTreeSet};

use toml::Value;
use zircon_ui::template::{UiAssetHeader, UiAssetImports, UiAssetRoot, UiBindingRef, UiChildMount};
use zircon_ui::template::{
    UiComponentDefinition, UiComponentParamSchema, UiNamedSlotSchema, UiNodeDefinition,
    UiNodeDefinitionKind, UiStyleDeclarationBlock, UiStyleRule, UiStyleScope, UiStyleSheet,
};
use zircon_ui::UiAssetDocument;

#[derive(Clone, Debug, PartialEq)]
struct UiAssetMapPatch<V> {
    key: String,
    next: Option<V>,
}

#[derive(Clone, Debug, PartialEq)]
struct UiAssetStructuredMapPatch<V, D> {
    key: String,
    next: Option<V>,
    diff: Option<D>,
}

#[derive(Clone, Debug, PartialEq)]
struct UiAssetIndexedVecPatch<V, D> {
    index: usize,
    next: V,
    diff: Option<D>,
}

#[derive(Clone, Debug, PartialEq)]
enum UiAssetKeyedVecOp<V, D> {
    Insert {
        index: usize,
        value: V,
    },
    Remove {
        key: String,
    },
    Move {
        key: String,
        index: usize,
    },
    Patch {
        key: String,
        next: V,
        diff: Option<D>,
    },
}

#[derive(Clone, Debug, PartialEq)]
enum UiAssetIndexedVecDiff<V, D> {
    Replace(Vec<V>),
    Patch {
        patches: Vec<UiAssetIndexedVecPatch<V, D>>,
        target: Vec<V>,
    },
    Keyed {
        operations: Vec<UiAssetKeyedVecOp<V, D>>,
        target: Vec<V>,
    },
}

#[derive(Clone, Debug, PartialEq)]
enum UiAssetOptionalMapDiff<V> {
    Replace(Option<BTreeMap<String, V>>),
    Patch {
        patches: Vec<UiAssetMapPatch<V>>,
        target: BTreeMap<String, V>,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct UiStyleDeclarationBlockDiff {
    self_values: Vec<UiAssetMapPatch<Value>>,
    slot: Vec<UiAssetMapPatch<Value>>,
}

impl UiStyleDeclarationBlockDiff {
    fn between(
        current: &UiStyleDeclarationBlock,
        target: &UiStyleDeclarationBlock,
    ) -> Option<Self> {
        let diff = Self {
            self_values: map_patches(&current.self_values, &target.self_values),
            slot: map_patches(&current.slot, &target.slot),
        };
        (!diff.self_values.is_empty() || !diff.slot.is_empty()).then_some(diff)
    }

    fn apply_to(&self, block: &mut UiStyleDeclarationBlock) -> bool {
        let mut changed = false;
        changed |= apply_map_patches(&mut block.self_values, &self.self_values);
        changed |= apply_map_patches(&mut block.slot, &self.slot);
        changed
    }
}

#[derive(Clone, Debug, PartialEq)]
struct UiStyleRuleDiff {
    selector: Option<String>,
    set: Option<UiStyleDeclarationBlockDiff>,
}

impl UiStyleRuleDiff {
    fn between(current: &UiStyleRule, target: &UiStyleRule) -> Option<Self> {
        let diff = Self {
            selector: scalar_patch(&current.selector, &target.selector),
            set: UiStyleDeclarationBlockDiff::between(&current.set, &target.set),
        };
        (diff.selector.is_some() || diff.set.is_some()).then_some(diff)
    }

    fn apply_to(&self, rule: &mut UiStyleRule) -> bool {
        let mut changed = false;
        changed |= apply_scalar_patch(&mut rule.selector, &self.selector);
        if let Some(set) = &self.set {
            changed |= set.apply_to(&mut rule.set);
        }
        changed
    }
}

#[derive(Clone, Debug, PartialEq)]
struct UiStyleSheetDiff {
    id: Option<String>,
    rules: Option<UiAssetIndexedVecDiff<UiStyleRule, UiStyleRuleDiff>>,
}

impl UiStyleSheetDiff {
    fn between(current: &UiStyleSheet, target: &UiStyleSheet) -> Option<Self> {
        let diff = Self {
            id: scalar_patch(&current.id, &target.id),
            rules: keyed_vec_diff(
                &current.rules,
                &target.rules,
                |rule| normalized_key(&rule.selector),
                UiStyleRuleDiff::between,
            ),
        };
        (diff.id.is_some() || diff.rules.is_some()).then_some(diff)
    }

    fn apply_to(&self, stylesheet: &mut UiStyleSheet) -> bool {
        let mut changed = false;
        changed |= apply_scalar_patch(&mut stylesheet.id, &self.id);
        if let Some(rules) = &self.rules {
            changed |= apply_indexed_vec_diff(
                &mut stylesheet.rules,
                rules,
                |rule| normalized_key(&rule.selector),
                |diff, rule| diff.apply_to(rule),
            );
        }
        changed
    }
}

#[derive(Clone, Debug, PartialEq)]
struct UiChildMountDiff {
    mount: Option<Option<String>>,
    slot: Vec<UiAssetMapPatch<Value>>,
}

impl UiChildMountDiff {
    fn between(current: &UiChildMount, target: &UiChildMount) -> Option<Self> {
        let diff = Self {
            mount: optional_patch(&current.mount, &target.mount),
            slot: map_patches(&current.slot, &target.slot),
        };
        (diff.mount.is_some() || !diff.slot.is_empty()).then_some(diff)
    }

    fn apply_to(&self, mount: &mut UiChildMount) -> bool {
        let mut changed = false;
        changed |= apply_optional_patch(&mut mount.mount, &self.mount);
        changed |= apply_map_patches(&mut mount.slot, &self.slot);
        changed
    }
}

#[derive(Clone, Debug, PartialEq)]
struct UiChildMountListDiff {
    target_order: Vec<String>,
    patches: Vec<UiAssetStructuredMapPatch<UiChildMount, UiChildMountDiff>>,
}

impl UiChildMountListDiff {
    fn between(current: &[UiChildMount], target: &[UiChildMount]) -> Option<Self> {
        if current == target {
            return None;
        }
        let current_map = current
            .iter()
            .cloned()
            .map(|mount| (mount.child.clone(), mount))
            .collect::<BTreeMap<_, _>>();
        let target_map = target
            .iter()
            .cloned()
            .map(|mount| (mount.child.clone(), mount))
            .collect::<BTreeMap<_, _>>();
        Some(Self {
            target_order: target.iter().map(|mount| mount.child.clone()).collect(),
            patches: structured_map_patches(&current_map, &target_map, UiChildMountDiff::between),
        })
    }

    fn apply_to(&self, children: &mut Vec<UiChildMount>) -> bool {
        let original = children.clone();
        let mut by_child = original
            .iter()
            .cloned()
            .map(|mount| (mount.child.clone(), mount))
            .collect::<BTreeMap<_, _>>();
        let changed = apply_structured_map_patches(&mut by_child, &self.patches, |diff, mount| {
            diff.apply_to(mount)
        });
        let mut rebuilt = Vec::new();
        let mut emitted = BTreeSet::new();
        for child_id in &self.target_order {
            if let Some(mount) = by_child.remove(child_id) {
                let _ = emitted.insert(child_id.clone());
                rebuilt.push(mount);
            }
        }
        for mount in original {
            if emitted.contains(&mount.child) {
                continue;
            }
            if let Some(preserved) = by_child.remove(&mount.child) {
                let _ = emitted.insert(mount.child.clone());
                rebuilt.push(preserved);
            }
        }
        for (child_id, mount) in by_child {
            if emitted.insert(child_id) {
                rebuilt.push(mount);
            }
        }
        if !changed && *children == rebuilt {
            return false;
        }
        *children = rebuilt;
        true
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct UiNodeDefinitionDiff {
    kind: Option<UiNodeDefinitionKind>,
    widget_type: Option<Option<String>>,
    component: Option<Option<String>>,
    component_ref: Option<Option<String>>,
    slot_name: Option<Option<String>>,
    control_id: Option<Option<String>>,
    classes: Option<Vec<String>>,
    params: Vec<UiAssetMapPatch<Value>>,
    props: Vec<UiAssetMapPatch<Value>>,
    layout: Option<UiAssetOptionalMapDiff<Value>>,
    bindings: Option<Vec<UiBindingRef>>,
    style_overrides: Option<UiStyleDeclarationBlockDiff>,
    children: Option<UiChildMountListDiff>,
}

impl UiNodeDefinitionDiff {
    fn between(current: &UiNodeDefinition, target: &UiNodeDefinition) -> Option<Self> {
        let diff = Self {
            kind: scalar_patch(&current.kind, &target.kind),
            widget_type: optional_patch(&current.widget_type, &target.widget_type),
            component: optional_patch(&current.component, &target.component),
            component_ref: optional_patch(&current.component_ref, &target.component_ref),
            slot_name: optional_patch(&current.slot_name, &target.slot_name),
            control_id: optional_patch(&current.control_id, &target.control_id),
            classes: vec_patch(&current.classes, &target.classes),
            params: map_patches(&current.params, &target.params),
            props: map_patches(&current.props, &target.props),
            layout: optional_map_patch(current.layout.as_ref(), target.layout.as_ref()),
            bindings: vec_patch(&current.bindings, &target.bindings),
            style_overrides: UiStyleDeclarationBlockDiff::between(
                &current.style_overrides,
                &target.style_overrides,
            ),
            children: UiChildMountListDiff::between(&current.children, &target.children),
        };
        diff.has_changes().then_some(diff)
    }

    fn apply_to(&self, node: &mut UiNodeDefinition) -> bool {
        let mut changed = false;
        changed |= apply_scalar_patch(&mut node.kind, &self.kind);
        changed |= apply_optional_patch(&mut node.widget_type, &self.widget_type);
        changed |= apply_optional_patch(&mut node.component, &self.component);
        changed |= apply_optional_patch(&mut node.component_ref, &self.component_ref);
        changed |= apply_optional_patch(&mut node.slot_name, &self.slot_name);
        changed |= apply_optional_patch(&mut node.control_id, &self.control_id);
        changed |= apply_vec_patch(&mut node.classes, &self.classes);
        changed |= apply_map_patches(&mut node.params, &self.params);
        changed |= apply_map_patches(&mut node.props, &self.props);
        changed |= apply_optional_map_patch(&mut node.layout, &self.layout);
        changed |= apply_vec_patch(&mut node.bindings, &self.bindings);
        if let Some(style_overrides) = &self.style_overrides {
            changed |= style_overrides.apply_to(&mut node.style_overrides);
        }
        if let Some(children) = &self.children {
            changed |= children.apply_to(&mut node.children);
        }
        changed
    }

    fn has_changes(&self) -> bool {
        self.kind.is_some()
            || self.widget_type.is_some()
            || self.component.is_some()
            || self.component_ref.is_some()
            || self.slot_name.is_some()
            || self.control_id.is_some()
            || self.classes.is_some()
            || !self.params.is_empty()
            || !self.props.is_empty()
            || self.layout.is_some()
            || self.bindings.is_some()
            || self.style_overrides.is_some()
            || self.children.is_some()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct UiComponentDefinitionDiff {
    root: Option<String>,
    style_scope: Option<UiStyleScope>,
    params: Vec<UiAssetMapPatch<UiComponentParamSchema>>,
    slots: Vec<UiAssetMapPatch<UiNamedSlotSchema>>,
}

impl UiComponentDefinitionDiff {
    fn between(current: &UiComponentDefinition, target: &UiComponentDefinition) -> Option<Self> {
        let diff = Self {
            root: scalar_patch(&current.root, &target.root),
            style_scope: scalar_patch(&current.style_scope, &target.style_scope),
            params: map_patches(&current.params, &target.params),
            slots: map_patches(&current.slots, &target.slots),
        };
        diff.has_changes().then_some(diff)
    }

    fn apply_to(&self, component: &mut UiComponentDefinition) -> bool {
        let mut changed = false;
        changed |= apply_scalar_patch(&mut component.root, &self.root);
        changed |= apply_scalar_patch(&mut component.style_scope, &self.style_scope);
        changed |= apply_map_patches(&mut component.params, &self.params);
        changed |= apply_map_patches(&mut component.slots, &self.slots);
        changed
    }

    fn has_changes(&self) -> bool {
        self.root.is_some()
            || self.style_scope.is_some()
            || !self.params.is_empty()
            || !self.slots.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct UiAssetDocumentDiff {
    asset: Option<UiAssetHeader>,
    imports: Option<UiAssetImports>,
    root: Option<Option<UiAssetRoot>>,
    tokens: Vec<UiAssetMapPatch<Value>>,
    nodes: Vec<UiAssetStructuredMapPatch<UiNodeDefinition, UiNodeDefinitionDiff>>,
    components: Vec<UiAssetStructuredMapPatch<UiComponentDefinition, UiComponentDefinitionDiff>>,
    stylesheets: Option<UiAssetIndexedVecDiff<UiStyleSheet, UiStyleSheetDiff>>,
}

impl UiAssetDocumentDiff {
    pub fn between(current: &UiAssetDocument, target: &UiAssetDocument) -> Self {
        Self {
            asset: (current.asset != target.asset).then(|| target.asset.clone()),
            imports: (current.imports != target.imports).then(|| target.imports.clone()),
            root: (current.root != target.root).then(|| target.root.clone()),
            tokens: map_patches(&current.tokens, &target.tokens),
            nodes: structured_map_patches(
                &current.nodes,
                &target.nodes,
                UiNodeDefinitionDiff::between,
            ),
            components: structured_map_patches(
                &current.components,
                &target.components,
                UiComponentDefinitionDiff::between,
            ),
            stylesheets: keyed_vec_diff(
                &current.stylesheets,
                &target.stylesheets,
                |sheet| normalized_key(&sheet.id),
                UiStyleSheetDiff::between,
            ),
        }
    }

    pub fn apply_to(&self, document: &mut UiAssetDocument) -> bool {
        let mut changed = false;

        if let Some(asset) = &self.asset {
            if document.asset != *asset {
                document.asset = asset.clone();
                changed = true;
            }
        }
        if let Some(imports) = &self.imports {
            if document.imports != *imports {
                document.imports = imports.clone();
                changed = true;
            }
        }
        if let Some(root) = &self.root {
            if document.root != *root {
                document.root = root.clone();
                changed = true;
            }
        }
        changed |= apply_map_patches(&mut document.tokens, &self.tokens);
        changed |= apply_structured_map_patches(&mut document.nodes, &self.nodes, |diff, node| {
            diff.apply_to(node)
        });
        changed |= apply_structured_map_patches(
            &mut document.components,
            &self.components,
            |diff, component| diff.apply_to(component),
        );
        if let Some(stylesheets) = &self.stylesheets {
            changed |= apply_indexed_vec_diff(
                &mut document.stylesheets,
                stylesheets,
                |sheet| normalized_key(&sheet.id),
                |diff, sheet| diff.apply_to(sheet),
            );
        }

        changed
    }
}

fn map_patches<V: Clone + PartialEq>(
    current: &BTreeMap<String, V>,
    target: &BTreeMap<String, V>,
) -> Vec<UiAssetMapPatch<V>> {
    current
        .keys()
        .chain(target.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(|key| {
            let current_value = current.get(&key);
            let target_value = target.get(&key);
            (current_value != target_value).then(|| UiAssetMapPatch {
                key,
                next: target_value.cloned(),
            })
        })
        .collect()
}

fn indexed_vec_diff<V: Clone + PartialEq, D>(
    current: &[V],
    target: &[V],
    diff_between: impl Fn(&V, &V) -> Option<D>,
) -> Option<UiAssetIndexedVecDiff<V, D>> {
    if current == target {
        return None;
    }
    if current.len() != target.len() {
        return Some(UiAssetIndexedVecDiff::Replace(target.to_vec()));
    }
    Some(UiAssetIndexedVecDiff::Patch {
        patches: current
            .iter()
            .zip(target.iter())
            .enumerate()
            .filter_map(|(index, (current_item, target_item))| {
                (current_item != target_item).then(|| UiAssetIndexedVecPatch {
                    index,
                    next: target_item.clone(),
                    diff: diff_between(current_item, target_item),
                })
            })
            .collect(),
        target: target.to_vec(),
    })
}

fn keyed_vec_diff<V: Clone + PartialEq, D>(
    current: &[V],
    target: &[V],
    key_of: impl Fn(&V) -> Option<String>,
    diff_between: impl Fn(&V, &V) -> Option<D>,
) -> Option<UiAssetIndexedVecDiff<V, D>> {
    if current == target {
        return None;
    }
    let Some(mut working_keys) = unique_keys(current, &key_of) else {
        return indexed_vec_diff(current, target, diff_between);
    };
    let Some(target_keys) = unique_keys(target, &key_of) else {
        return Some(UiAssetIndexedVecDiff::Replace(target.to_vec()));
    };

    let mut working_values = current.to_vec();
    let mut operations = Vec::new();

    for (target_index, target_item) in target.iter().enumerate() {
        let target_key = &target_keys[target_index];
        match working_keys
            .iter()
            .position(|existing_key| existing_key == target_key)
        {
            Some(current_index) => {
                if current_index != target_index {
                    let moved_key = working_keys.remove(current_index);
                    let moved_value = working_values.remove(current_index);
                    let insert_index = target_index.min(working_keys.len());
                    working_keys.insert(insert_index, moved_key.clone());
                    working_values.insert(insert_index, moved_value);
                    operations.push(UiAssetKeyedVecOp::Move {
                        key: moved_key,
                        index: insert_index,
                    });
                }
                if working_values[target_index] != *target_item {
                    let diff = diff_between(&working_values[target_index], target_item);
                    working_values[target_index] = target_item.clone();
                    operations.push(UiAssetKeyedVecOp::Patch {
                        key: target_key.clone(),
                        next: target_item.clone(),
                        diff,
                    });
                }
            }
            None => {
                let insert_index = target_index.min(working_keys.len());
                working_keys.insert(insert_index, target_key.clone());
                working_values.insert(insert_index, target_item.clone());
                operations.push(UiAssetKeyedVecOp::Insert {
                    index: insert_index,
                    value: target_item.clone(),
                });
            }
        }
    }

    while working_keys.len() > target_keys.len() {
        let key = working_keys.pop().expect("working key");
        let _ = working_values.pop();
        operations.push(UiAssetKeyedVecOp::Remove { key });
    }

    Some(UiAssetIndexedVecDiff::Keyed {
        operations,
        target: target.to_vec(),
    })
}

fn structured_map_patches<V: Clone + PartialEq, D>(
    current: &BTreeMap<String, V>,
    target: &BTreeMap<String, V>,
    diff_between: impl Fn(&V, &V) -> Option<D>,
) -> Vec<UiAssetStructuredMapPatch<V, D>> {
    current
        .keys()
        .chain(target.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(|key| match (current.get(&key), target.get(&key)) {
            (Some(current_value), Some(target_value)) if current_value != target_value => {
                Some(UiAssetStructuredMapPatch {
                    key,
                    next: Some(target_value.clone()),
                    diff: diff_between(current_value, target_value),
                })
            }
            (None, Some(target_value)) => Some(UiAssetStructuredMapPatch {
                key,
                next: Some(target_value.clone()),
                diff: None,
            }),
            (Some(_), None) => Some(UiAssetStructuredMapPatch {
                key,
                next: None,
                diff: None,
            }),
            _ => None,
        })
        .collect()
}

fn apply_indexed_vec_diff<V: Clone + PartialEq, D>(
    values: &mut Vec<V>,
    diff: &UiAssetIndexedVecDiff<V, D>,
    key_of: impl Fn(&V) -> Option<String>,
    apply_nested_diff: impl Fn(&D, &mut V) -> bool,
) -> bool {
    match diff {
        UiAssetIndexedVecDiff::Replace(next) => {
            if values != next {
                *values = next.clone();
                true
            } else {
                false
            }
        }
        UiAssetIndexedVecDiff::Patch { patches, target } => {
            if values.len() != target.len() {
                if values != target {
                    *values = target.clone();
                    return true;
                }
                return false;
            }
            let mut changed = false;
            for patch in patches {
                let Some(item) = values.get_mut(patch.index) else {
                    if values != target {
                        *values = target.clone();
                        return true;
                    }
                    return false;
                };
                changed |= match &patch.diff {
                    Some(diff) => apply_nested_diff(diff, item),
                    None if item != &patch.next => {
                        *item = patch.next.clone();
                        true
                    }
                    None => false,
                };
            }
            changed
        }
        UiAssetIndexedVecDiff::Keyed { operations, target } => {
            let mut changed = false;
            for operation in operations {
                match operation {
                    UiAssetKeyedVecOp::Insert { index, value } => {
                        let insert_index = (*index).min(values.len());
                        values.insert(insert_index, value.clone());
                        changed = true;
                    }
                    UiAssetKeyedVecOp::Remove { key } => {
                        let Some(index) = values
                            .iter()
                            .position(|item| key_of(item).as_deref() == Some(key.as_str()))
                        else {
                            if values != target {
                                *values = target.clone();
                                return true;
                            }
                            return false;
                        };
                        values.remove(index);
                        changed = true;
                    }
                    UiAssetKeyedVecOp::Move { key, index } => {
                        let Some(current_index) = values
                            .iter()
                            .position(|item| key_of(item).as_deref() == Some(key.as_str()))
                        else {
                            if values != target {
                                *values = target.clone();
                                return true;
                            }
                            return false;
                        };
                        let moved_value = values.remove(current_index);
                        let insert_index = (*index).min(values.len());
                        let move_changed = current_index != insert_index;
                        values.insert(insert_index, moved_value);
                        changed |= move_changed;
                    }
                    UiAssetKeyedVecOp::Patch { key, next, diff } => {
                        let Some(current_index) = values
                            .iter()
                            .position(|item| key_of(item).as_deref() == Some(key.as_str()))
                        else {
                            if values != target {
                                *values = target.clone();
                                return true;
                            }
                            return false;
                        };
                        let item = &mut values[current_index];
                        changed |= match diff {
                            Some(diff) => apply_nested_diff(diff, item),
                            None if item != next => {
                                *item = next.clone();
                                true
                            }
                            None => false,
                        };
                    }
                }
            }
            changed
        }
    }
}

fn unique_keys<V>(items: &[V], key_of: impl Fn(&V) -> Option<String>) -> Option<Vec<String>> {
    let mut seen = BTreeSet::new();
    let mut keys = Vec::with_capacity(items.len());
    for item in items {
        let key = key_of(item)?;
        if !seen.insert(key.clone()) {
            return None;
        }
        keys.push(key);
    }
    Some(keys)
}

fn normalized_key(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn apply_map_patches<V: Clone + PartialEq>(
    values: &mut BTreeMap<String, V>,
    patches: &[UiAssetMapPatch<V>],
) -> bool {
    let mut changed = false;
    for patch in patches {
        match &patch.next {
            Some(next) => {
                if values.get(&patch.key) != Some(next) {
                    let _ = values.insert(patch.key.clone(), next.clone());
                    changed = true;
                }
            }
            None => {
                if values.remove(&patch.key).is_some() {
                    changed = true;
                }
            }
        }
    }
    changed
}

fn apply_structured_map_patches<V: Clone + PartialEq, D>(
    values: &mut BTreeMap<String, V>,
    patches: &[UiAssetStructuredMapPatch<V, D>],
    apply_diff: impl Fn(&D, &mut V) -> bool,
) -> bool {
    let mut changed = false;
    for patch in patches {
        match (&patch.next, &patch.diff) {
            (Some(next), Some(diff)) => match values.get_mut(&patch.key) {
                Some(existing) => {
                    changed |= apply_diff(diff, existing);
                }
                None => {
                    let _ = values.insert(patch.key.clone(), next.clone());
                    changed = true;
                }
            },
            (Some(next), None) => {
                if values.get(&patch.key) != Some(next) {
                    let _ = values.insert(patch.key.clone(), next.clone());
                    changed = true;
                }
            }
            (None, _) => {
                if values.remove(&patch.key).is_some() {
                    changed = true;
                }
            }
        }
    }
    changed
}

fn scalar_patch<V: Clone + PartialEq>(current: &V, target: &V) -> Option<V> {
    (current != target).then(|| target.clone())
}

fn apply_scalar_patch<V: Clone + PartialEq>(field: &mut V, patch: &Option<V>) -> bool {
    let Some(next) = patch else {
        return false;
    };
    if field != next {
        *field = next.clone();
        true
    } else {
        false
    }
}

fn optional_patch<V: Clone + PartialEq>(
    current: &Option<V>,
    target: &Option<V>,
) -> Option<Option<V>> {
    (current != target).then(|| target.clone())
}

fn apply_optional_patch<V: Clone + PartialEq>(
    field: &mut Option<V>,
    patch: &Option<Option<V>>,
) -> bool {
    let Some(next) = patch else {
        return false;
    };
    if field != next {
        *field = next.clone();
        true
    } else {
        false
    }
}

fn vec_patch<V: Clone + PartialEq>(current: &[V], target: &[V]) -> Option<Vec<V>> {
    (current != target).then(|| target.to_vec())
}

fn apply_vec_patch<V: Clone + PartialEq>(field: &mut Vec<V>, patch: &Option<Vec<V>>) -> bool {
    let Some(next) = patch else {
        return false;
    };
    if field != next {
        *field = next.clone();
        true
    } else {
        false
    }
}

fn optional_map_patch<V: Clone + PartialEq>(
    current: Option<&BTreeMap<String, V>>,
    target: Option<&BTreeMap<String, V>>,
) -> Option<UiAssetOptionalMapDiff<V>> {
    match (current, target) {
        (Some(current_map), Some(target_map)) if current_map != target_map => {
            Some(UiAssetOptionalMapDiff::Patch {
                patches: map_patches(current_map, target_map),
                target: target_map.clone(),
            })
        }
        (None, None) => None,
        _ if current != target => Some(UiAssetOptionalMapDiff::Replace(target.cloned())),
        _ => None,
    }
}

fn apply_optional_map_patch<V: Clone + PartialEq>(
    field: &mut Option<BTreeMap<String, V>>,
    patch: &Option<UiAssetOptionalMapDiff<V>>,
) -> bool {
    let Some(patch) = patch else {
        return false;
    };
    match patch {
        UiAssetOptionalMapDiff::Replace(next) => {
            if field != next {
                *field = next.clone();
                true
            } else {
                false
            }
        }
        UiAssetOptionalMapDiff::Patch { patches, target } => match field.as_mut() {
            Some(values) => apply_map_patches(values, patches),
            None => {
                *field = Some(target.clone());
                true
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use zircon_ui::template::{
        UiChildMount, UiComponentParamSchema, UiNamedSlotSchema, UiStyleScope,
    };
    use zircon_ui::{UiAssetKind, template::UiNodeDefinitionKind};

    use super::*;

    #[test]
    fn node_diff_preserves_unrelated_existing_node_fields() {
        let before = node_diff_fixture_document();
        let mut after = before.clone();
        let button = after.nodes.get_mut("button").expect("button node");
        button.control_id = Some("ToolbarSave".to_string());
        button
            .layout
            .as_mut()
            .expect("button layout")
            .insert("width".to_string(), Value::Integer(120));

        let diff = UiAssetDocumentDiff::between(&before, &after);

        let mut diverged = before.clone();
        diverged.nodes.insert(
            "badge".to_string(),
            UiNodeDefinition {
                kind: UiNodeDefinitionKind::Native,
                widget_type: Some("Label".to_string()),
                component: None,
                component_ref: None,
                slot_name: None,
                control_id: Some("Badge".to_string()),
                classes: Vec::new(),
                params: BTreeMap::new(),
                props: BTreeMap::from([("text".to_string(), Value::String("Draft".to_string()))]),
                layout: None,
                bindings: Vec::new(),
                style_overrides: Default::default(),
                children: Vec::new(),
            },
        );
        let button = diverged.nodes.get_mut("button").expect("diverged button");
        button.props.insert(
            "icon".to_string(),
            Value::String("asset://icons/save".to_string()),
        );
        button
            .layout
            .as_mut()
            .expect("diverged button layout")
            .insert("max".to_string(), Value::Integer(240));
        button.children.push(UiChildMount {
            child: "badge".to_string(),
            mount: None,
            slot: BTreeMap::new(),
        });

        assert!(diff.apply_to(&mut diverged));

        let button = diverged.nodes.get("button").expect("patched button");
        assert_eq!(button.control_id.as_deref(), Some("ToolbarSave"));
        assert_eq!(
            button
                .layout
                .as_ref()
                .and_then(|layout| layout.get("width")),
            Some(&Value::Integer(120))
        );
        assert_eq!(
            button.props.get("icon").and_then(Value::as_str),
            Some("asset://icons/save")
        );
        assert_eq!(
            button.layout.as_ref().and_then(|layout| layout.get("max")),
            Some(&Value::Integer(240))
        );
        assert_eq!(button.children.len(), 1);
        assert_eq!(button.children[0].child, "badge");
    }

    #[test]
    fn component_diff_preserves_unrelated_existing_component_fields() {
        let before = component_diff_fixture_document();
        let mut after = before.clone();
        after
            .components
            .get_mut("Card")
            .expect("component")
            .style_scope = UiStyleScope::Open;

        let diff = UiAssetDocumentDiff::between(&before, &after);

        let mut diverged = before.clone();
        let component = diverged
            .components
            .get_mut("Card")
            .expect("diverged component");
        component.params.insert(
            "icon".to_string(),
            UiComponentParamSchema {
                r#type: "resource".to_string(),
                default: None,
            },
        );
        component.slots.insert(
            "footer".to_string(),
            UiNamedSlotSchema {
                required: false,
                multiple: true,
            },
        );

        assert!(diff.apply_to(&mut diverged));

        let component = diverged.components.get("Card").expect("patched component");
        assert_eq!(component.style_scope, UiStyleScope::Open);
        assert!(component.params.contains_key("icon"));
        assert!(component.slots.contains_key("footer"));
    }

    #[test]
    fn node_diff_preserves_existing_child_mounts_when_children_change() {
        let before = node_diff_fixture_document();
        let mut after = before.clone();
        after.nodes.insert(
            "spacer".to_string(),
            UiNodeDefinition {
                kind: UiNodeDefinitionKind::Native,
                widget_type: Some("Space".to_string()),
                component: None,
                component_ref: None,
                slot_name: None,
                control_id: Some("Spacer".to_string()),
                classes: Vec::new(),
                params: BTreeMap::new(),
                props: BTreeMap::new(),
                layout: None,
                bindings: Vec::new(),
                style_overrides: Default::default(),
                children: Vec::new(),
            },
        );
        after
            .nodes
            .get_mut("root")
            .expect("root")
            .children
            .push(UiChildMount {
                child: "spacer".to_string(),
                mount: None,
                slot: BTreeMap::new(),
            });

        let diff = UiAssetDocumentDiff::between(&before, &after);

        let mut diverged = before.clone();
        diverged.nodes.insert(
            "badge".to_string(),
            UiNodeDefinition {
                kind: UiNodeDefinitionKind::Native,
                widget_type: Some("Label".to_string()),
                component: None,
                component_ref: None,
                slot_name: None,
                control_id: Some("Badge".to_string()),
                classes: Vec::new(),
                params: BTreeMap::new(),
                props: BTreeMap::new(),
                layout: None,
                bindings: Vec::new(),
                style_overrides: Default::default(),
                children: Vec::new(),
            },
        );
        let root = diverged.nodes.get_mut("root").expect("diverged root");
        root.children[0]
            .slot
            .insert("padding".to_string(), Value::Integer(8));
        root.children.push(UiChildMount {
            child: "badge".to_string(),
            mount: None,
            slot: BTreeMap::new(),
        });

        assert!(diff.apply_to(&mut diverged));

        let root = diverged.nodes.get("root").expect("patched root");
        assert_eq!(
            root.children
                .iter()
                .map(|child| child.child.as_str())
                .collect::<Vec<_>>(),
            vec!["button", "spacer", "badge"]
        );
        assert_eq!(
            root.children[0].slot.get("padding"),
            Some(&Value::Integer(8))
        );
    }

    #[test]
    fn stylesheet_diff_preserves_unrelated_existing_rules_when_one_rule_changes() {
        let before = stylesheet_diff_fixture_document();
        let mut after = before.clone();
        after.stylesheets[0].rules[0].selector = ".toolbar > Button.primary".to_string();
        after.stylesheets[0].rules[0].set.self_values.insert(
            "background.color".to_string(),
            Value::String("#4488ff".to_string()),
        );

        let diff = UiAssetDocumentDiff::between(&before, &after);

        let mut diverged = before.clone();
        diverged.stylesheets[1].rules[0].selector = ".secondary:hover".to_string();
        diverged.stylesheets[1].rules[1].set.self_values.insert(
            "text.color".to_string(),
            Value::String("#999999".to_string()),
        );

        assert!(diff.apply_to(&mut diverged));

        assert_eq!(
            diverged.stylesheets[0].rules[0].selector,
            ".toolbar > Button.primary"
        );
        assert_eq!(
            diverged.stylesheets[0].rules[0]
                .set
                .self_values
                .get("background.color"),
            Some(&Value::String("#4488ff".to_string()))
        );
        assert_eq!(
            diverged.stylesheets[1].rules[0].selector,
            ".secondary:hover"
        );
        assert_eq!(
            diverged.stylesheets[1].rules[1]
                .set
                .self_values
                .get("text.color"),
            Some(&Value::String("#999999".to_string()))
        );
    }

    #[test]
    fn stylesheet_diff_preserves_unrelated_existing_stylesheets_when_sheets_reorder_insert_and_remove(
    ) {
        let before = stylesheet_diff_fixture_document();
        let mut after = before.clone();
        after.stylesheets[1].rules[0].selector = ".secondary:focus".to_string();
        after.stylesheets = vec![
            after.stylesheets[1].clone(),
            UiStyleSheet {
                id: "palette".to_string(),
                rules: vec![UiStyleRule {
                    selector: "#palette_panel".to_string(),
                    set: UiStyleDeclarationBlock {
                        self_values: BTreeMap::from([(
                            "background.color".to_string(),
                            Value::String("#171717".to_string()),
                        )]),
                        slot: BTreeMap::new(),
                    },
                }],
            },
        ];

        let diff = UiAssetDocumentDiff::between(&before, &after);

        let mut diverged = before.clone();
        diverged.stylesheets[1].rules[1]
            .set
            .self_values
            .insert("margin".to_string(), Value::Integer(4));
        diverged.stylesheets[0].rules[0].selector = ".toolbar > Button:pressed".to_string();

        assert!(diff.apply_to(&mut diverged));

        assert_eq!(
            diverged
                .stylesheets
                .iter()
                .map(|sheet| sheet.id.as_str())
                .collect::<Vec<_>>(),
            vec!["inspector", "palette"]
        );
        assert_eq!(
            diverged.stylesheets[0].rules[0].selector,
            ".secondary:focus"
        );
        assert_eq!(
            diverged.stylesheets[0].rules[1]
                .set
                .self_values
                .get("margin"),
            Some(&Value::Integer(4))
        );
        assert_eq!(
            diverged.stylesheets[1].rules[0]
                .set
                .self_values
                .get("background.color"),
            Some(&Value::String("#171717".to_string()))
        );
    }

    #[test]
    fn stylesheet_diff_preserves_unrelated_existing_rules_when_rules_reorder_insert_and_remove() {
        let before = stylesheet_diff_fixture_document();
        let mut after = before.clone();
        let hover_rule = after.stylesheets[0].rules[1].clone();
        after.stylesheets[0].rules = vec![
            hover_rule,
            UiStyleRule {
                selector: ".toolbar > Button.primary".to_string(),
                set: UiStyleDeclarationBlock {
                    self_values: BTreeMap::from([(
                        "background.color".to_string(),
                        Value::String("#3355aa".to_string()),
                    )]),
                    slot: BTreeMap::new(),
                },
            },
        ];

        let diff = UiAssetDocumentDiff::between(&before, &after);

        let mut diverged = before.clone();
        diverged.stylesheets[0].rules[1]
            .set
            .self_values
            .insert("font.weight".to_string(), Value::String("bold".to_string()));
        diverged.stylesheets[0].rules[0].set.self_values.insert(
            "border.color".to_string(),
            Value::String("#111111".to_string()),
        );

        assert!(diff.apply_to(&mut diverged));

        let rules = &diverged.stylesheets[0].rules;
        assert_eq!(
            rules
                .iter()
                .map(|rule| rule.selector.as_str())
                .collect::<Vec<_>>(),
            vec![".toolbar > Button:hover", ".toolbar > Button.primary"]
        );
        assert_eq!(
            rules[0].set.self_values.get("font.weight"),
            Some(&Value::String("bold".to_string()))
        );
        assert_eq!(
            rules[1].set.self_values.get("background.color"),
            Some(&Value::String("#3355aa".to_string()))
        );
    }

    fn node_diff_fixture_document() -> UiAssetDocument {
        UiAssetDocument {
            asset: UiAssetHeader {
                kind: UiAssetKind::Layout,
                id: "editor.test.node_diff".to_string(),
                version: 1,
                display_name: "Node Diff".to_string(),
            },
            imports: UiAssetImports::default(),
            tokens: BTreeMap::new(),
            root: Some(UiAssetRoot {
                node: "root".to_string(),
            }),
            nodes: BTreeMap::from([
                (
                    "root".to_string(),
                    UiNodeDefinition {
                        kind: UiNodeDefinitionKind::Native,
                        widget_type: Some("VerticalBox".to_string()),
                        component: None,
                        component_ref: None,
                        slot_name: None,
                        control_id: Some("Root".to_string()),
                        classes: Vec::new(),
                        params: BTreeMap::new(),
                        props: BTreeMap::new(),
                        layout: None,
                        bindings: Vec::new(),
                        style_overrides: Default::default(),
                        children: vec![UiChildMount {
                            child: "button".to_string(),
                            mount: None,
                            slot: BTreeMap::new(),
                        }],
                    },
                ),
                (
                    "button".to_string(),
                    UiNodeDefinition {
                        kind: UiNodeDefinitionKind::Native,
                        widget_type: Some("Button".to_string()),
                        component: None,
                        component_ref: None,
                        slot_name: None,
                        control_id: Some("SaveButton".to_string()),
                        classes: vec!["primary".to_string()],
                        params: BTreeMap::new(),
                        props: BTreeMap::from([(
                            "text".to_string(),
                            Value::String("Save".to_string()),
                        )]),
                        layout: Some(BTreeMap::from([("width".to_string(), Value::Integer(100))])),
                        bindings: Vec::new(),
                        style_overrides: Default::default(),
                        children: Vec::new(),
                    },
                ),
            ]),
            components: BTreeMap::new(),
            stylesheets: Vec::new(),
        }
    }

    fn component_diff_fixture_document() -> UiAssetDocument {
        UiAssetDocument {
            asset: UiAssetHeader {
                kind: UiAssetKind::Widget,
                id: "editor.test.component_diff".to_string(),
                version: 1,
                display_name: "Component Diff".to_string(),
            },
            imports: UiAssetImports::default(),
            tokens: BTreeMap::new(),
            root: Some(UiAssetRoot {
                node: "card_root".to_string(),
            }),
            nodes: BTreeMap::new(),
            components: BTreeMap::from([(
                "Card".to_string(),
                UiComponentDefinition {
                    root: "card_root".to_string(),
                    style_scope: UiStyleScope::Closed,
                    params: BTreeMap::from([(
                        "title".to_string(),
                        UiComponentParamSchema {
                            r#type: "string".to_string(),
                            default: Some(Value::String("Card".to_string())),
                        },
                    )]),
                    slots: BTreeMap::from([(
                        "body".to_string(),
                        UiNamedSlotSchema {
                            required: true,
                            multiple: false,
                        },
                    )]),
                },
            )]),
            stylesheets: Vec::new(),
        }
    }

    fn stylesheet_diff_fixture_document() -> UiAssetDocument {
        UiAssetDocument {
            asset: UiAssetHeader {
                kind: UiAssetKind::Layout,
                id: "editor.test.stylesheet_diff".to_string(),
                version: 1,
                display_name: "Stylesheet Diff".to_string(),
            },
            imports: UiAssetImports::default(),
            tokens: BTreeMap::new(),
            root: None,
            nodes: BTreeMap::new(),
            components: BTreeMap::new(),
            stylesheets: vec![
                UiStyleSheet {
                    id: "editor_shell".to_string(),
                    rules: vec![
                        zircon_ui::template::UiStyleRule {
                            selector: ".toolbar > Button".to_string(),
                            set: UiStyleDeclarationBlock {
                                self_values: BTreeMap::from([(
                                    "background.color".to_string(),
                                    Value::String("#223344".to_string()),
                                )]),
                                slot: BTreeMap::new(),
                            },
                        },
                        zircon_ui::template::UiStyleRule {
                            selector: ".toolbar > Button:hover".to_string(),
                            set: UiStyleDeclarationBlock {
                                self_values: BTreeMap::from([(
                                    "text.color".to_string(),
                                    Value::String("#ffffff".to_string()),
                                )]),
                                slot: BTreeMap::new(),
                            },
                        },
                    ],
                },
                UiStyleSheet {
                    id: "inspector".to_string(),
                    rules: vec![
                        zircon_ui::template::UiStyleRule {
                            selector: ".secondary".to_string(),
                            set: UiStyleDeclarationBlock {
                                self_values: BTreeMap::from([(
                                    "background.color".to_string(),
                                    Value::String("#101010".to_string()),
                                )]),
                                slot: BTreeMap::new(),
                            },
                        },
                        zircon_ui::template::UiStyleRule {
                            selector: "#inspector_panel".to_string(),
                            set: UiStyleDeclarationBlock {
                                self_values: BTreeMap::from([(
                                    "padding".to_string(),
                                    Value::Integer(12),
                                )]),
                                slot: BTreeMap::new(),
                            },
                        },
                    ],
                },
            ],
        }
    }
}

