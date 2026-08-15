mod atomic_file;
mod error;
mod resource_io;
pub(crate) mod transaction;

pub use atomic_file::atomic_write;
#[cfg(test)]
pub(crate) use atomic_file::NEXT_ATOMIC_FILE_ID;
pub(crate) use atomic_file::{
    atomic_write_with_fault, ensure_parent_directories, is_atomic_write_transaction_path,
    recover_missing_target_from_backup, replace_staged_file, stage_atomic_write,
    sync_parent_directory, AtomicWriteFault,
};
pub use error::ResourceIoError;
pub use resource_io::ResourceIo;
