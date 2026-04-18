use std::collections::HashSet;

use zircon_resource::ResourceManager;

use super::project_locators;
use crate::{AssetUri, ProjectManager};

pub(in crate::pipeline::manager) fn clear_removed_project_resources(
    resource_manager: &ResourceManager,
    previous_locators: &HashSet<AssetUri>,
    project: &ProjectManager,
) {
    let current = project_locators(project);
    for locator in previous_locators.difference(&current) {
        let _ = resource_manager.remove_by_locator(locator);
    }
}
