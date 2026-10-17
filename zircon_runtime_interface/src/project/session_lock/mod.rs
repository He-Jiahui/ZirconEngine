//! Shared project-session record and namespace contract.
//!
//! Platform hosts may implement the operating-system lease differently, but they must use this
//! record format and, on Windows, this namespace identity to address the same editor session.
//!
//! 此模块只定义共享记录、V2 文本编解码、记录路径和 Windows 互斥命名规则；
//! 具体平台租约由宿主持有，记录内容单独存在时不能证明租约仍有效。

#[path = "codec.rs"]
mod codec;
#[path = "error.rs"]
mod error;
#[path = "identity.rs"]
mod identity;
#[path = "record.rs"]
mod record;

pub use codec::{
    decode_project_session_admission_record, encode_project_session_admission_record,
    MAX_PROJECT_SESSION_ADMISSION_RECORD_BYTES,
};
pub use error::ProjectSessionAdmissionRecordError;
#[cfg(windows)]
pub use identity::windows_project_session_mutex_name;
pub use identity::{project_session_lock_path, PROJECT_SESSION_LOCK_FILE_NAME};
pub use record::{
    ProjectSessionAdmissionLifecycleV1, ProjectSessionAdmissionRecordV1,
    ProjectSessionGenerationV1, ProjectSessionPrincipalV1,
    MAX_PROJECT_SESSION_ADMISSION_INSTANCE_ID_BYTES,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
