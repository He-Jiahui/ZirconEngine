use std::collections::HashSet;

use crate::asset::AssetUri;
use crate::asset::project::ProjectManager;

pub(in crate::asset::pipeline::manager) fn project_locators(
    project: &ProjectManager,
) -> HashSet<AssetUri> {
    project
        .registry()
        .values()
        .map(|metadata| metadata.primary_locator().clone())
        .collect()
}
