#[path = "artifact_identity.rs"]
mod artifact_identity;
#[path = "atomic_file/mod.rs"]
pub(crate) mod atomic_file;
#[path = "transaction/mod.rs"]
pub(crate) mod transaction;

#[cfg(test)]
pub(crate) use artifact_identity::next_test_output_id;
pub use artifact_identity::ArtifactIdentityExhausted;
pub use atomic_file::{atomic_write, atomic_write_new};
pub(crate) use atomic_file::{
    ensure_parent_directories, is_atomic_write_transaction_path,
    publish_staged_file_for_transaction, replace_staged_file, sync_parent_directory,
};
