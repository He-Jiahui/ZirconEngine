use std::collections::HashSet;

use crate::{AssetUri, ProjectManager};

pub(in crate::pipeline::manager) fn project_locators(
    project: &ProjectManager,
) -> HashSet<AssetUri> {
    project
        .registry()
        .values()
        .map(|metadata| metadata.primary_locator().clone())
        .collect()
}
