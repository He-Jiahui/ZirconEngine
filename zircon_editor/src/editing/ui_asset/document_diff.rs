use std::collections::BTreeSet;

use toml::Value;
use zircon_ui::{
    UiAssetDocument, UiAssetHeader, UiAssetImports, UiAssetRoot, UiComponentDefinition,
    UiNodeDefinition, UiStyleSheet,
};

#[derive(Clone, Debug, PartialEq)]
struct UiAssetMapPatch<V> {
    key: String,
    next: Option<V>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct UiAssetDocumentDiff {
    asset: Option<UiAssetHeader>,
    imports: Option<UiAssetImports>,
    root: Option<Option<UiAssetRoot>>,
    tokens: Vec<UiAssetMapPatch<Value>>,
    nodes: Vec<UiAssetMapPatch<UiNodeDefinition>>,
    components: Vec<UiAssetMapPatch<UiComponentDefinition>>,
    stylesheets: Option<Vec<UiStyleSheet>>,
}

impl UiAssetDocumentDiff {
    pub fn between(current: &UiAssetDocument, target: &UiAssetDocument) -> Self {
        Self {
            asset: (current.asset != target.asset).then(|| target.asset.clone()),
            imports: (current.imports != target.imports).then(|| target.imports.clone()),
            root: (current.root != target.root).then(|| target.root.clone()),
            tokens: map_patches(&current.tokens, &target.tokens),
            nodes: map_patches(&current.nodes, &target.nodes),
            components: map_patches(&current.components, &target.components),
            stylesheets: (current.stylesheets != target.stylesheets)
                .then(|| target.stylesheets.clone()),
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
        changed |= apply_map_patches(&mut document.nodes, &self.nodes);
        changed |= apply_map_patches(&mut document.components, &self.components);
        if let Some(stylesheets) = &self.stylesheets {
            if document.stylesheets != *stylesheets {
                document.stylesheets = stylesheets.clone();
                changed = true;
            }
        }

        changed
    }
}

fn map_patches<V: Clone + PartialEq>(
    current: &std::collections::BTreeMap<String, V>,
    target: &std::collections::BTreeMap<String, V>,
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

fn apply_map_patches<V: Clone + PartialEq>(
    values: &mut std::collections::BTreeMap<String, V>,
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
