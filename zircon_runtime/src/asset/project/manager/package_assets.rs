//! 包资产根注册更新当前项目的解析目录与 catalog 元数据；应在依赖扫描前完成，使 package:// 引用采用确定的根。

use std::path::Path;

use crate::asset::AssetImportError;

use super::ProjectManager;

impl ProjectManager {
    pub fn register_package_asset_root(
        &mut self,
        package_id: impl Into<String>,
        assets_root: impl AsRef<Path>,
    ) -> Result<(), AssetImportError> {
        let mut package_assets = std::mem::take(&mut self.package_assets);
        if let Err(error) = package_assets.register_root(package_id, assets_root) {
            self.package_assets = package_assets;
            return Err(error);
        }
        let catalog_input_generation =
            super::super::ProjectCatalogInputGeneration::publish_metadata(
                &self.catalog_input_generation,
                self.paths.root(),
                &self.manifest,
                &package_assets,
            );
        self.package_assets = package_assets;
        self.catalog_input_generation = catalog_input_generation;
        Ok(())
    }

    pub fn register_package_asset_roots<Root>(
        &mut self,
        package_id: impl Into<String>,
        asset_roots: impl IntoIterator<Item = Root>,
        package_root: impl AsRef<Path>,
    ) -> Result<(), AssetImportError>
    where
        Root: AsRef<str>,
    {
        let mut package_assets = std::mem::take(&mut self.package_assets);
        if let Err(error) =
            package_assets.register_package_roots(package_id, asset_roots, package_root)
        {
            self.package_assets = package_assets;
            return Err(error);
        }
        let catalog_input_generation =
            super::super::ProjectCatalogInputGeneration::publish_metadata(
                &self.catalog_input_generation,
                self.paths.root(),
                &self.manifest,
                &package_assets,
            );
        self.package_assets = package_assets;
        self.catalog_input_generation = catalog_input_generation;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/package_assets_performance_tests.rs"]
mod performance_tests;
