use std::collections::HashSet;

use crate::core::resource::ResourceMutationBatch;

use super::project_locators;
use crate::asset::project::ProjectManager;
use crate::asset::AssetUri;

pub(in crate::asset::pipeline::manager) fn clear_removed_project_resources(
    mut batch: ResourceMutationBatch,
    previous_locators: &HashSet<AssetUri>,
    project: &ProjectManager,
) -> ResourceMutationBatch {
    let current = project_locators(project);
    for locator in previous_locators.difference(&current) {
        batch = batch.remove(locator.clone());
    }
    batch
}
