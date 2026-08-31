mod cover;
mod create_project;
mod create_project_request;
mod device_install;
mod install_receipt;
mod local_paths;
mod metadata;
mod package;
mod recent_project;
mod recycle_bin;
mod shared_recent_projects;
mod validation;

pub use cover::project_cover_path;
pub use create_project::{create_project, CreateProjectError, CreateProjectReport};
pub use create_project_request::{
    project_template_catalog, CreateProjectRequest, CreateProjectRequestError, ProjectTemplate,
    ProjectTemplateInfo,
};
pub use device_install::{install_package_to_device, DeviceInstallReport, DeviceInstallRequest};
pub use install_receipt::{
    DeviceInstallFileReceipt, DeviceInstallReceipt, HubContentDownloadChunk,
    HubContentDownloadManifest,
};
pub use metadata::{
    metadata_for_path, metadata_for_path_mut, normalize_project_root, project_filesystem_path_key,
    project_metadata_key, project_paths_match, prune_empty_metadata, ProjectMetadata,
    ProjectMetadataMap,
};
pub use package::{package_project, ProjectPackageReport, ProjectPackageRequest};
pub use recent_project::{now_unix_ms, RecentProject, RECENT_PROJECT_LIMIT};
pub use recycle_bin::{recycle_delete_project, RecycleDeleteCommand};
pub use shared_recent_projects::{
    load_shared_recent_projects, load_shared_recent_projects_snapshot,
    merge_recent_project_entries, reconcile_shared_recent_projects,
    reconcile_shared_recent_projects_snapshot, SharedRecentProjectsError,
    SharedRecentProjectsSnapshot,
};
pub use validation::{validate_project_root, ProjectValidation};
