//! Editor-side admission views and job receipts for asset refactor commands.
//!
//! These views read the Runtime registry authority. Filesystem mutation and reference rewrites
//! remain owned by the Runtime mutation transaction.

#[path = "delete.rs"]
mod delete;
#[path = "deletion.rs"]
mod deletion;
#[path = "folder.rs"]
mod folder;
#[path = "reimport.rs"]
mod reimport;
#[path = "relocation.rs"]
mod relocation;

pub use delete::{AssetDeleteDisposition, AssetDeletePreflight};
pub use deletion::{EditorAssetDeletionResult, EditorAssetDeletionTicket};
pub use folder::{EditorAssetFolderOperation, EditorAssetFolderResult, EditorAssetFolderTicket};
pub use reimport::{EditorAssetReimportResult, EditorAssetReimportTicket};
pub use relocation::{EditorAssetRelocationResult, EditorAssetRelocationTicket};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
