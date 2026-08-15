//! Shared recent-project registry DTO and deterministic identity rules.
//!
//! This module contains no filesystem or operating-system locking behavior. Hosts use the
//! storage and mutex identities here, then retain ownership of the corresponding I/O lease.

mod entry;
mod error;
mod identity;
mod registry;
mod storage_path;

pub use entry::HubRecentProjectV1;
pub use error::HubRecentProjectsError;
pub use identity::hub_recent_project_path_key;
#[cfg(windows)]
pub use identity::windows_hub_recent_projects_mutex_name;
pub use registry::{merge_hub_recent_projects, HubRecentProjectsV1, HUB_RECENT_PROJECT_LIMIT_V1};
pub use storage_path::{
    hub_recent_projects_lock_path, hub_recent_projects_path, hub_recent_projects_path_from_home,
};
