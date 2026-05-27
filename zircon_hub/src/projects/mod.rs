mod cover;
mod create_project_request;
mod device_install;
mod editor_recent_sync;
mod metadata;
mod package;
mod recent_project;
mod recycle_bin;
mod validation;

pub use cover::project_cover_path;
pub use create_project_request::{
    project_template_catalog, CreateProjectRequest, ProjectTemplate, ProjectTemplateInfo,
};
pub use device_install::{install_package_to_device, DeviceInstallReport, DeviceInstallRequest};
pub use editor_recent_sync::{
    load_editor_recent_project_session, load_editor_recent_projects, merge_recent_projects,
    save_editor_recent_projects, save_editor_recent_projects_with_last_project,
    EditorRecentProjectSession,
};
pub use metadata::{
    metadata_for_path, metadata_for_path_mut, project_filesystem_path_key, project_metadata_key,
    project_paths_match, prune_empty_metadata, ProjectMetadata, ProjectMetadataMap,
};
pub use package::{package_project, ProjectPackageReport, ProjectPackageRequest};
pub use recent_project::{now_unix_ms, RecentProject, RECENT_PROJECT_LIMIT};
pub use recycle_bin::{recycle_delete_project, RecycleDeleteCommand};
pub use validation::{validate_project_root, ProjectValidation};
