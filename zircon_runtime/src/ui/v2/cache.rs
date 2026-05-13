use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use zircon_runtime_interface::ui::v2::{UiV2AssetDocument, UiV2AssetError};

#[derive(Clone, Debug, Default)]
pub struct UiV2PrototypeStore {
    assets: BTreeMap<String, Arc<UiV2AssetDocument>>,
}

impl UiV2PrototypeStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, document: UiV2AssetDocument) -> Arc<UiV2AssetDocument> {
        let asset_id = document.asset.id.clone();
        let document = Arc::new(document);
        let _ = self.assets.insert(asset_id, Arc::clone(&document));
        document
    }

    pub fn insert_alias(&mut self, asset_id: impl Into<String>, document: Arc<UiV2AssetDocument>) {
        let _ = self.assets.insert(asset_id.into(), document);
    }

    pub fn get(&self, asset_id: &str) -> Option<Arc<UiV2AssetDocument>> {
        self.assets.get(asset_id).map(Arc::clone)
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct UiV2PrototypeStoreBuilder {
    store: UiV2PrototypeStore,
    declared_assets: BTreeSet<String>,
}

impl UiV2PrototypeStoreBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, document: UiV2AssetDocument) -> Arc<UiV2AssetDocument> {
        self.insert_with_aliases(document, std::iter::empty::<String>())
    }

    pub fn insert_with_aliases<I, S>(
        &mut self,
        document: UiV2AssetDocument,
        aliases: I,
    ) -> Arc<UiV2AssetDocument>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for reference in &document.imports.widgets {
            if let Some((asset_id, _)) = reference.split_once('#') {
                let _ = self.declared_assets.insert(asset_id.to_string());
            }
        }
        for reference in &document.imports.styles {
            let _ = self.declared_assets.insert(reference.clone());
        }
        let document = self.store.insert(document);
        let canonical_id = document.asset.id.as_str();
        for alias in aliases {
            let alias = alias.into();
            if alias != canonical_id {
                self.store.insert_alias(alias, Arc::clone(&document));
            }
        }
        document
    }

    pub fn build(self) -> Result<UiV2PrototypeStore, UiV2AssetError> {
        for asset_id in self.declared_assets {
            if self.store.get(&asset_id).is_none() {
                return Err(UiV2AssetError::InvalidDocument {
                    asset_id,
                    detail: "declared UI v2 import is not loaded in the prototype store"
                        .to_string(),
                });
            }
        }
        Ok(self.store)
    }
}
