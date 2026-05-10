use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use zircon_runtime_interface::ui::template::{UiAssetError, UiRawAssetPrototype};

#[derive(Clone, Debug, Default)]
pub struct UiPrototypeStore {
    assets: BTreeMap<String, Arc<UiRawAssetPrototype>>,
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
        let (asset_id, component_name) =
            reference
                .split_once('#')
                .ok_or_else(|| UiAssetError::InvalidDocument {
                    asset_id: reference.to_string(),
                    detail: "component references must include a #Component suffix".to_string(),
                })?;
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
            if let Some((asset_id, _)) = reference.split_once('#') {
                let _ = self.declared_assets.insert(asset_id.to_string());
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
