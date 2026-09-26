//! Hub 与 Editor 共用的最近项目投影及文件事务。
//!
//! 此模块定义记录格式、词法路径键和带平台锁的读改写；项目身份与打开权限仍由宿主预检决定。

mod entry;
mod error;
mod identity;
mod registry;
mod storage_path;
mod store;

pub use entry::HubRecentProjectV1;
pub use error::HubRecentProjectsError;
pub use identity::hub_recent_project_path_key;
#[cfg(windows)]
pub use identity::windows_hub_recent_projects_mutex_name;
pub use registry::{
    merge_hub_recent_projects, HubRecentProjectTombstoneV1, HubRecentProjectsV1,
    HUB_RECENT_PROJECT_LIMIT_V1, HUB_RECENT_PROJECT_TOMBSTONE_LIMIT_V1,
};
pub use storage_path::{
    hub_recent_projects_lock_path, hub_recent_projects_path, hub_recent_projects_path_from_home,
};
pub use store::{
    HubRecentProjectsLoad, HubRecentProjectsLoadDisposition, HubRecentProjectsMutation,
    HubRecentProjectsStore, HubRecentProjectsStoreError, HubRecentProjectsWritePolicy,
    HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1,
};
