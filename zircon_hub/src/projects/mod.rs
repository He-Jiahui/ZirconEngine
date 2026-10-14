#[path = "cloud_binding.rs"]
mod cloud_binding;
#[path = "cover.rs"]
mod cover;
#[path = "create_project_request.rs"]
mod create_project_request;
#[path = "device_install.rs"]
mod device_install;
#[path = "install_receipt.rs"]
mod install_receipt;
#[path = "local_paths.rs"]
mod local_paths;
#[path = "metadata.rs"]
mod metadata;
#[path = "package.rs"]
mod package;
#[path = "recent_project.rs"]
mod recent_project;
#[path = "recycle_bin.rs"]
mod recycle_bin;
#[path = "shared_recent_projects.rs"]
mod shared_recent_projects;
#[path = "validation.rs"]
mod validation;

pub use cloud_binding::{CloudAccountScope, CloudBindingEnvironment, CloudProjectBinding};
pub use cover::project_cover_path;
pub use create_project_request::{
    CreateProjectRequest, CreateProjectRequestError, ProjectTemplateInfo,
    enabled_project_template_id, project_template_catalog,
};
pub use device_install::{DeviceInstallReport, DeviceInstallRequest, install_package_to_device};
pub use install_receipt::{
    DeviceInstallFileReceipt, DeviceInstallReceipt, HubContentDownloadChunk,
    HubContentDownloadManifest,
};
pub use metadata::{
    ProjectMetadata, ProjectMetadataMap, metadata_for_path, metadata_for_path_mut,
    normalize_project_root, project_filesystem_path_key, project_metadata_key, project_paths_match,
    prune_empty_metadata,
};
pub use package::{ProjectPackageReport, ProjectPackageRequest, package_project};
pub use recent_project::{RECENT_PROJECT_LIMIT, RecentProject, now_unix_ms};
pub use recycle_bin::{RecycleDeleteCommand, recycle_delete_project};
pub use shared_recent_projects::{
    SharedRecentProjectsError, SharedRecentProjectsSnapshot, load_shared_recent_projects,
    load_shared_recent_projects_snapshot, merge_recent_project_entries,
    reconcile_shared_recent_projects, reconcile_shared_recent_projects_snapshot,
};
pub use validation::{ProjectValidation, validate_project_root};
pub use zircon_runtime_interface::project::ProjectTemplateId;
