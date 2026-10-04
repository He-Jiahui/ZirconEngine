use crate::asset::ProjectAssetManager;
use crate::plugin::RuntimeExtensionRegistryError;

use super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    /// 将本轮目录中的导入器安装到项目资源管理器；调用方须管理目标管理器的重复安装及卸载生命周期。
    pub fn apply_asset_importers_to_project_asset_manager(
        &mut self,
        manager: &ProjectAssetManager,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.finalize();
        for importer in self.asset_importers().importers() {
            manager
                .register_asset_importer_arc(importer)
                .map_err(|error| RuntimeExtensionRegistryError::AssetImporter(error.to_string()))?;
        }
        Ok(())
    }
}
