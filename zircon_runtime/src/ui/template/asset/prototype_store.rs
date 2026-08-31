use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use zircon_runtime_interface::ui::template::{
    parse_component_reference, UiAssetError, UiRawAssetPrototype,
};

#[cfg(test)]
#[path = "prototype_store/hash_index_tests.rs"]
mod hash_index_tests;

#[derive(Clone, Debug, Default)]
pub struct UiPrototypeStore {
    assets: HashMap<String, Arc<UiRawAssetPrototype>>,
}

impl UiPrototypeStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, prototype: UiRawAssetPrototype) -> Arc<UiRawAssetPrototype> {
        let asset_id = prototype.asset.id.clone();
        let prototype = Arc::new(prototype);
        let _ = self.assets.insert(asset_id, Arc::clone(&prototype));
        prototype
    }

    pub fn insert_alias(
        &mut self,
        asset_id: impl Into<String>,
        prototype: Arc<UiRawAssetPrototype>,
    ) {
        let _ = self.assets.insert(asset_id.into(), prototype);
    }

    pub fn get(&self, asset_id: &str) -> Option<Arc<UiRawAssetPrototype>> {
        self.assets.get(asset_id).map(Arc::clone)
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    pub fn component_prototype(
        &self,
        reference: &str,
    ) -> Result<(Arc<UiRawAssetPrototype>, String), UiAssetError> {
        let (asset_id, component_name) = parse_component_reference(reference)?;
        let asset = self
            .get(asset_id)
            .ok_or_else(|| UiAssetError::UnknownImport {
                reference: reference.to_string(),
            })?;
        if !asset.components.contains_key(component_name) {
            return Err(UiAssetError::UnknownComponent {
                asset_id: asset.asset.id.clone(),
                component: component_name.to_string(),
            });
        }
        Ok((asset, component_name.to_string()))
    }
}

#[derive(Clone, Debug, Default)]
pub struct UiPrototypeStoreBuilder {
    store: UiPrototypeStore,
    declared_assets: BTreeSet<String>,
    invalid_widget_import: Option<UiAssetError>,
}

impl UiPrototypeStoreBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, prototype: UiRawAssetPrototype) -> Arc<UiRawAssetPrototype> {
        self.insert_with_aliases(prototype, std::iter::empty::<String>())
    }

    pub fn insert_with_aliases<I, S>(
        &mut self,
        prototype: UiRawAssetPrototype,
        aliases: I,
    ) -> Arc<UiRawAssetPrototype>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for reference in &prototype.imports.widgets {
            if reference.contains('#') {
                match parse_component_reference(reference) {
                    Ok((asset_id, _)) => {
                        let _ = self.declared_assets.insert(asset_id.to_string());
                    }
                    Err(error) => {
                        let _ = self.invalid_widget_import.get_or_insert(error);
                    }
                }
            } else {
                let _ = self.declared_assets.insert(reference.clone());
            }
        }
        for reference in &prototype.imports.styles {
            let _ = self.declared_assets.insert(reference.clone());
        }
        let prototype = self.store.insert(prototype);
        let canonical_id = prototype.asset.id.as_str();
        for alias in aliases {
            let alias = alias.into();
            if alias != canonical_id {
                self.store.insert_alias(alias, Arc::clone(&prototype));
            }
        }
        prototype
    }

    pub fn build(self) -> Result<UiPrototypeStore, UiAssetError> {
        if let Some(error) = self.invalid_widget_import {
            return Err(error);
        }
        for asset_id in self.declared_assets {
            if self.store.get(&asset_id).is_none() {
                return Err(UiAssetError::UnknownImport {
                    reference: asset_id,
                });
            }
        }
        Ok(self.store)
    }
}
