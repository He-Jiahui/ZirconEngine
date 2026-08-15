//! Versioned DTOs for the Hub-to-Editor file-mailbox handshake.

mod focus_signal;
mod focus_signal_path;
mod mailbox;
mod outcome;
mod protocol_version;
mod recent_projects;
mod session_token;

#[cfg(test)]
mod tests;

pub use focus_signal::HubEditorFocusSignalV1;
pub use focus_signal_path::{hub_editor_focus_signal_path, HubEditorFocusSignalPathError};
pub use mailbox::HubEditorMailboxV1;
pub use outcome::HubEditorLaunchOutcomeV1;
pub use protocol_version::{HubProtocolVersionV1, HUB_PROTOCOL_VERSION_V1};
#[cfg(windows)]
pub use recent_projects::windows_hub_recent_projects_mutex_name;
pub use recent_projects::{
    hub_recent_project_path_key, hub_recent_projects_lock_path, hub_recent_projects_path,
    hub_recent_projects_path_from_home, merge_hub_recent_projects, HubRecentProjectV1,
    HubRecentProjectsError, HubRecentProjectsV1, HUB_RECENT_PROJECT_LIMIT_V1,
};
pub use session_token::{HubSessionToken, HubSessionTokenParseError};
