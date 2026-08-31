use std::path::Path;

use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime_interface::hub_protocol::{
    HubRecentProjectV1, HubRecentProjectsMutation, HubRecentProjectsStore, HubRecentProjectsV1,
    HubRecentProjectsWritePolicy, hub_recent_projects_path,
};
use zircon_runtime_interface::project::ProjectManifestSummary;

pub(crate) use zircon_runtime_interface::hub_protocol::HubRecentProjectsStoreError;

/// Reads the rebuildable Hub history projection without making Welcome dependent on its health.
pub(crate) fn load_recent_projects() -> Result<HubRecentProjectsV1, HubRecentProjectsStoreError> {
    HubRecentProjectsStore::new(hub_recent_projects_path())
        .load_projection()
        .map(|load| load.registry().clone())
}

/// Records a post-Ready projection without waiting for another process to release its lease.
pub(crate) fn record_recent_project(
    project_root: &Path,
    summary: ProjectManifestSummary,
    last_opened_unix_ms: u64,
) -> Result<HubRecentProjectsV1, HubRecentProjectsStoreError> {
    let registry_path = hub_recent_projects_path();
    let display_path = ProjectPaths::resolve_path(project_root)
        .map(|resolved| resolved.display_path().to_path_buf())
        .map_err(|source| HubRecentProjectsStoreError::Io {
            operation: "resolve project root for shared recent-project registry",
            path: project_root.to_path_buf(),
            source,
        })?;
    let project =
        HubRecentProjectV1::new(summary, display_path, last_opened_unix_ms).map_err(|source| {
            HubRecentProjectsStoreError::Contract {
                path: registry_path,
                source,
            }
        })?;
    mutate_recent_projects(|registry| registry.record(project))
}

/// Removes a projection entry without waiting on the shared writer lease from the UI thread.
pub(crate) fn forget_recent_project(
    project_root: &Path,
) -> Result<HubRecentProjectsV1, HubRecentProjectsStoreError> {
    let display_path = ProjectPaths::resolve_path(project_root)
        .map(|resolved| resolved.display_path().to_path_buf())
        .unwrap_or_else(|_| project_root.to_path_buf());
    mutate_recent_projects(|registry| registry.remove(display_path))
}

fn mutate_recent_projects(
    update: impl FnOnce(
        &mut HubRecentProjectsV1,
    )
        -> Result<(), zircon_runtime_interface::hub_protocol::HubRecentProjectsError>,
) -> Result<HubRecentProjectsV1, HubRecentProjectsStoreError> {
    let result: HubRecentProjectsMutation = HubRecentProjectsStore::new(hub_recent_projects_path())
        .update(HubRecentProjectsWritePolicy::try_now(), update)?;
    Ok(result.registry().clone())
}
