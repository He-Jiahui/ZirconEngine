use std::collections::HashSet;

use crate::asset::project::ProjectManager;
use crate::asset::AssetUri;

// 收集项目注册表中的主定位符，供项目切换清理旧归属并在关闭时生成移除变化。
pub(in crate::asset::pipeline::manager) fn project_locators(
    project: &ProjectManager,
) -> HashSet<AssetUri> {
    project
        .registry()
        .values()
        .map(|metadata| metadata.primary_locator().clone())
        .collect()
}
