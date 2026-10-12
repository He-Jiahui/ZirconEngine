#[path = "command_flow.rs"]
mod command_flow;
#[path = "crash_windows.rs"]
mod crash_windows;
#[path = "../../project_commandlet/document_migration.rs"]
mod document_migration;
#[path = "resolver_index.rs"]
mod resolver_index;
#[path = "scale_acceptance.rs"]
mod scale_acceptance;
#[path = "source_boundary.rs"]
mod source_boundary;
#[path = "../../project_commandlet/transaction_recovery.rs"]
mod transaction_recovery;

use std::fs;

use crate::asset::project::ProjectPaths;

use crate::asset::migration::{
    migrate_project_assets, migrate_project_assets_with_commit_fault,
    migrate_project_assets_with_commit_window_fault,
    migrate_project_assets_with_process_interruption, migrate_project_assets_with_restore_fault,
    migrate_project_assets_with_rollback_cleanup_fault, migrate_project_assets_with_stage_fault,
    migrate_project_assets_with_terminal_interruption, AssetMigrationIssueKind, AssetMigrationMode,
    AssetMigrationOptions, AssetMigrationTransactionPhase,
};
use crate::asset::{
    AssetKind, AssetReference, AssetUri, AssetUuid, ReferenceResolutionError, ZMaterialDocument,
};

use super::{fixture_root, write_manifest, write_registered_source, write_registered_subasset};

fn directory_snapshot(path: &std::path::Path) -> Vec<(std::ffi::OsString, Vec<u8>)> {
    let mut snapshot = fs::read_dir(path)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (entry.file_name(), fs::read(entry.path()).unwrap())
        })
        .collect::<Vec<_>>();
    snapshot.sort_by(|left, right| left.0.cmp(&right.0));
    snapshot
}
