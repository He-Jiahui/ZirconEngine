use std::collections::HashSet;

use crate::core::resource::ResourceMutationBatch;

use crate::asset::project::ProjectManager;
use crate::asset::AssetUri;

/// 项目代次更新后只退役旧项目拥有、且新目录已不再声明的资源；调用方再合并导入更新统一提交。
pub(in crate::asset::pipeline::manager) fn clear_removed_project_resources(
    mut batch: ResourceMutationBatch,
    previous_locators: &HashSet<AssetUri>,
    project: &ProjectManager,
) -> ResourceMutationBatch {
    let current = collect_project_locator_refs(
        project
            .registry()
            .values()
            .map(|metadata| metadata.primary_locator()),
    );
    for locator in previous_locators {
        if !current.contains(locator) {
            batch = batch.remove(locator.clone());
        }
    }
    batch
}

fn collect_project_locator_refs<'a>(
    locators: impl IntoIterator<Item = &'a AssetUri>,
) -> HashSet<&'a AssetUri> {
    locators.into_iter().collect()
}

#[cfg(test)]
#[path = "tests/clear_removed_project_resources.rs"]
mod tests;
