//! Shared project-session record and namespace contract.
//!
//! Platform hosts may implement the operating-system lease differently, but they must use this
//! record format and, on Windows, this namespace identity to address the same editor session.

mod codec;
mod error;
mod identity;
mod record;

pub use codec::{decode_project_session_lock_record, encode_project_session_lock_record};
pub use error::ProjectSessionLockRecordDecodeError;
#[cfg(windows)]
pub use identity::windows_project_session_mutex_name;
pub use identity::{project_session_lock_path, PROJECT_SESSION_LOCK_FILE_NAME};
pub use record::ProjectSessionLockRecordV1;

#[cfg(test)]
mod tests;
