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
pub use record::{SessionLockInspection, SessionLockRecord, SESSION_LOCK_FILE_NAME};

use mutation::{create_lock, remove_lock, replace_lock};
#[cfg(all(test, windows))]
pub(super) use ownership_lease::session_mutex_name_for_test;
use ownership_lease::SessionOwnershipLease;
use record::{
    encode_record, inspect_lock, new_record, read_lock, session_lock_directory, session_lock_path,
    unix_millis,
};
