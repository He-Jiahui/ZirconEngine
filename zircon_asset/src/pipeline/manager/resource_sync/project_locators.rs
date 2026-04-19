use std::collections::HashSet;

use crate::project::ProjectManager;
use crate::AssetUri;

pub(in crate::pipeline::manager) fn project_locators(
    project: &ProjectManager,
) -> HashSet<AssetUri> {
    project
        .registry()
        .values()
        .map(|metadata| metadata.primary_locator().clone())
        .collect()
}
