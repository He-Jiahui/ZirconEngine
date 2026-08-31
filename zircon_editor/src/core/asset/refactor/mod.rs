//! Editor-side admission views for asset refactor commands.
//!
//! These views read the runtime registry authority. Filesystem mutation, redirectors, and
//! reference rewrites belong to the later runtime mutation transaction rather than this module.

mod delete;
mod deletion;
mod relocation;

pub use delete::{AssetDeleteDisposition, AssetDeletePreflight};
pub use deletion::{EditorAssetDeletionResult, EditorAssetDeletionTicket};
pub use relocation::{EditorAssetRelocationResult, EditorAssetRelocationTicket};

#[cfg(test)]
mod tests;
