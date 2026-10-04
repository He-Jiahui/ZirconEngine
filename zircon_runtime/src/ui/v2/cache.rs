//! 原型仓库存放创建表面前已加载的文档，供组件展开与跨资源样式解析共享读取。

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;

use zircon_runtime_interface::ui::v2::{UiV2AssetDocument, UiV2AssetError};

use super::component_reference::{parse_v2_widget_import_reference, UiV2WidgetImportReference};

pub fn source_path_identity_for_path(path: &Path) -> Option<String> {
    let canonical_path = path.canonicalize().ok()?;
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?;
    let canonical_workspace_root = workspace_root.canonicalize().ok()?;
    let relative = canonical_path
        .strip_prefix(&canonical_workspace_root)
        .ok()?;
    let components = relative
        .components()
        .map(|component| component.as_os_str().to_str().map(str::to_string))
        .collect::<Option<Vec<_>>>()?;
    (!components.is_empty()).then(|| components.join("/"))
}

#[cfg(test)]
#[path = "cache/tests/hash_lookup_tests.rs"]
mod hash_lookup_tests;

/// 同一文档可登记为规范身份及路径别名；读取返回共享所有权，不进行文件 I/O。
/// 枚举按登记键排序，别名可能使同一文档出现多次，不能将键数量当作唯一文档数量。
#[derive(Clone, Debug, Default)]
pub struct UiV2PrototypeStore {
    assets: BTreeMap<String, Arc<UiV2AssetDocument>>,
    asset_lookup: HashMap<String, Arc<UiV2AssetDocument>>,
    source_paths: BTreeMap<String, String>,
    ambiguous_source_paths: BTreeSet<String>,
}

impl UiV2PrototypeStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, document: UiV2AssetDocument) -> Arc<UiV2AssetDocument> {
        let asset_id = document.asset.id.clone();
        let document = Arc::new(document);
        let _ = self.assets.insert(asset_id.clone(), Arc::clone(&document));
        let _ = self.asset_lookup.insert(asset_id, Arc::clone(&document));
        document
    }

    pub fn insert_with_source_path(
        &mut self,
        document: UiV2AssetDocument,
        source_path: impl Into<String>,
    ) -> Arc<UiV2AssetDocument> {
        let document = self.insert(document);
        let asset_id = document.asset.id.clone();
        let source_path = source_path.into();
        if self.ambiguous_source_paths.contains(&asset_id) {
            return document;
        }
        if let Some(existing) = self.source_paths.get(&asset_id) {
            if existing != &source_path {
                let _ = self.source_paths.remove(&asset_id);
                let _ = self.ambiguous_source_paths.insert(asset_id);
                return document;
            }
        } else {
            let _ = self.source_paths.insert(asset_id, source_path);
        }
        document
    }

    /// 加载器用别名把路径引用连接到文档身份；替换别名不会同步替换已有的其他登记键。
    pub fn insert_alias(&mut self, asset_id: impl Into<String>, document: Arc<UiV2AssetDocument>) {
        let asset_id = asset_id.into();
        let _ = self.assets.insert(asset_id.clone(), Arc::clone(&document));
        let _ = self.asset_lookup.insert(asset_id, document);
    }

    pub fn get(&self, asset_id: &str) -> Option<Arc<UiV2AssetDocument>> {
        self.asset_lookup.get(asset_id).map(Arc::clone)
    }

    pub fn source_path_for_asset_id(&self, asset_id: &str) -> Option<&str> {
        let document = self.asset_lookup.get(asset_id)?;
        self.source_paths
            .get(&document.asset.id)
            .map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    pub fn documents(&self) -> impl Iterator<Item = Arc<UiV2AssetDocument>> + '_ {
        self.assets.values().cloned()
    }
}

#[derive(Clone, Debug, Default)]
pub struct UiV2PrototypeStoreBuilder {
    store: UiV2PrototypeStore,
    declared_assets: BTreeSet<String>,
    invalid_widget_import: Option<UiV2AssetError>,
}

impl UiV2PrototypeStoreBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, document: UiV2AssetDocument) -> Arc<UiV2AssetDocument> {
        self.insert_with_aliases(document, std::iter::empty::<String>())
    }

    pub fn insert_with_source_path(
        &mut self,
        document: UiV2AssetDocument,
        source_path: impl Into<String>,
    ) -> Arc<UiV2AssetDocument> {
        self.insert_with_aliases_and_source_path(
            document,
            std::iter::empty::<String>(),
            source_path,
        )
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
        self.insert_with_aliases_inner(document, aliases, None)
    }

    pub fn insert_with_aliases_and_source_path<I, S>(
        &mut self,
        document: UiV2AssetDocument,
        aliases: I,
        source_path: impl Into<String>,
    ) -> Arc<UiV2AssetDocument>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.insert_with_aliases_inner(document, aliases, Some(source_path.into()))
    }

    fn insert_with_aliases_inner<I, S>(
        &mut self,
        document: UiV2AssetDocument,
        aliases: I,
        source_path: Option<String>,
    ) -> Arc<UiV2AssetDocument>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for reference in &document.imports.widgets {
            match parse_v2_widget_import_reference(&document.asset.id, reference) {
                Ok(UiV2WidgetImportReference::WholeAsset(asset_id))
                | Ok(UiV2WidgetImportReference::Component { asset_id, .. }) => {
                    let _ = self.declared_assets.insert(asset_id.to_string());
                }
                Err(error) => {
                    let _ = self.invalid_widget_import.get_or_insert(error);
                }
            }
        }
        for reference in &document.imports.styles {
            let _ = self.declared_assets.insert(reference.clone());
        }
        let document = if let Some(source_path) = source_path {
            self.store.insert_with_source_path(document, source_path)
        } else {
            self.store.insert(document)
        };
        let canonical_id = document.asset.id.as_str();
        for alias in aliases {
            let alias = alias.into();
            if alias != canonical_id {
                self.store.insert_alias(alias, Arc::clone(&document));
            }
        }
        document
    }

    /// 全仓库入口：先插入所有导入资源，再在这里一次性拒绝缺失导入和错误引用语法。
    pub fn build(self) -> Result<UiV2PrototypeStore, UiV2AssetError> {
        if let Some(error) = self.invalid_widget_import {
            return Err(error);
        }
        Self::validate_declared_assets(&self.store, self.declared_assets)?;
        Ok(self.store)
    }

    /// Validates only the documents reachable from explicit roots. Runtime project
    /// loading uses this so an unreferenced editor or experimental `.zui` asset
    /// cannot prevent an otherwise valid project UI surface from starting.
    /// 仅验证根文档的导入闭包，返回的仓库仍保留全部已加载文档；组件名称在展开阶段验证。
    pub fn build_for_roots<I, S>(self, roots: I) -> Result<UiV2PrototypeStore, UiV2AssetError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut pending = roots
            .into_iter()
            .map(|root| root.as_ref().to_string())
            .collect::<Vec<_>>();
        let mut visited = BTreeSet::new();

        while let Some(asset_id) = pending.pop() {
            let Some(document) = self.store.get(&asset_id) else {
                return Err(Self::missing_import(asset_id));
            };
            if !visited.insert(document.asset.id.clone()) {
                continue;
            }
            for reference in &document.imports.widgets {
                match parse_v2_widget_import_reference(&document.asset.id, reference)? {
                    UiV2WidgetImportReference::WholeAsset(asset_id)
                    | UiV2WidgetImportReference::Component { asset_id, .. } => {
                        pending.push(asset_id.to_string());
                    }
                }
            }
            pending.extend(document.imports.styles.iter().cloned());
        }
        Ok(self.store)
    }

    fn validate_declared_assets(
        store: &UiV2PrototypeStore,
        declared_assets: BTreeSet<String>,
    ) -> Result<(), UiV2AssetError> {
        for asset_id in declared_assets {
            if store.get(&asset_id).is_none() {
                return Err(Self::missing_import(asset_id));
            }
        }
        Ok(())
    }

    fn missing_import(asset_id: String) -> UiV2AssetError {
        UiV2AssetError::InvalidDocument {
            asset_id,
            detail: "declared UI v2 import is not loaded in the prototype store".to_string(),
        }
    }
}
