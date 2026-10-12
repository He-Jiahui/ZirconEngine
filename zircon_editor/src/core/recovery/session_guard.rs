#[path = "session_guard/admission.rs"]
mod admission;
#[path = "session_guard/durability.rs"]
mod durability;
#[path = "session_guard/error.rs"]
mod error;
#[path = "session_guard/guard.rs"]
mod guard;
#[path = "session_guard/liveness.rs"]
mod liveness;
#[path = "session_guard/mutation.rs"]
mod mutation;
#[path = "session_guard/ownership_lease.rs"]
mod ownership_lease;
#[path = "session_guard/record.rs"]
mod record;

pub use durability::SessionLockDurability;
pub use error::SessionGuardError;
pub use guard::SessionGuard;
pub use liveness::{SessionGuardAdmission, SessionGuardResidual};
pub use record::{ProjectSessionAdmissionRecordV1, SessionLockInspection, SESSION_LOCK_FILE_NAME};

pub use admission::SessionAdmissionRequest;
use mutation::{create_lock, remove_lock, replace_lock};
#[cfg(all(test, windows))]
pub(super) use ownership_lease::session_mutex_name_for_test;
use ownership_lease::SessionOwnershipLease;
use record::{
    encode_record, inspect_lock, new_record, next_session_generation, read_lock,
    session_lock_directory, session_lock_path, unix_millis,
};
