use crate::asset::pipeline::manager::{project_asset_manager_handle, ProjectAssetManager};
use crate::core::manager::resolve_manager_service;
use crate::core::{CoreError, CoreResult, ModuleContext, ModuleLifecycle};

use super::PROJECT_ASSET_MANAGER_NAME;

#[derive(Debug, Default)]
pub(super) struct AssetModuleLifecycle;

// 模块 ready 表示项目目录代次可被一致读取；具体资产载荷的就绪状态由资源 readiness 单独查询。
impl ModuleLifecycle for AssetModuleLifecycle {
    fn ready(&self, context: &ModuleContext) -> CoreResult<bool> {
        let core = context
            .core
            .upgrade()
            .ok_or_else(|| CoreError::ServiceUnavailable(PROJECT_ASSET_MANAGER_NAME.to_owned()))?;
        let manager: std::sync::Arc<ProjectAssetManager> =
            resolve_manager_service(&core, project_asset_manager_handle(&core)?)?;
        Ok(manager.catalog_generation_is_ready())
    }
}
