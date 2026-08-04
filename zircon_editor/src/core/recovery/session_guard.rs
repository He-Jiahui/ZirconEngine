mod durability;
mod error;
mod guard;
mod mutation;
mod ownership_lease;
mod record;

pub use durability::SessionLockDurability;
pub use error::SessionGuardError;
pub use guard::SessionGuard;
pub use record::{SESSION_LOCK_FILE_NAME, SessionLockInspection, SessionLockRecord};

pub(super) use mutation::{create_lock, remove_lock, replace_lock};
pub(super) use ownership_lease::SessionOwnershipLease;
#[cfg(all(test, windows))]
pub(super) use ownership_lease::session_mutex_name_for_test;
pub(super) use record::{
    encode_record, inspect_lock, new_record, read_lock, session_lock_directory, session_lock_path,
    unix_millis,
};
