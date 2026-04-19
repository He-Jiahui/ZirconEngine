use std::collections::HashSet;

use crate::core::resource::ResourceManager;

use super::project_locators;
use crate::asset::project::ProjectManager;
use crate::asset::AssetUri;

pub(in crate::asset::pipeline::manager) fn clear_removed_project_resources(
    resource_manager: &ResourceManager,
    previous_locators: &HashSet<AssetUri>,
    project: &ProjectManager,
) {
    let current = project_locators(project);
    for locator in previous_locators.difference(&current) {
        let _ = resource_manager.remove_by_locator(locator);
    }
}
