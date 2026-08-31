mod admission;
mod durability;
mod error;
mod guard;
mod liveness;
mod mutation;
mod ownership_lease;
mod record;

pub use durability::SessionLockDurability;
pub use error::SessionGuardError;
pub use guard::SessionGuard;
pub use liveness::{SessionGuardAdmission, SessionGuardResidual};
pub use record::{ProjectSessionAdmissionRecordV1, SESSION_LOCK_FILE_NAME, SessionLockInspection};

pub use admission::SessionAdmissionRequest;
use mutation::{create_lock, remove_lock, replace_lock};
use ownership_lease::SessionOwnershipLease;
#[cfg(all(test, windows))]
pub(super) use ownership_lease::session_mutex_name_for_test;
use record::{
    encode_record, inspect_lock, new_record, next_session_generation, read_lock,
    session_lock_directory, session_lock_path, unix_millis,
};
